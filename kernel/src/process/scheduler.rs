/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2025 Gavin Kellam

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

use core::fmt::Display;

use super::{
    Process, ProcessId, RefProcess, WeakProcess,
    task::Task,
    thread::{RefThread, WeakThread},
};
use crate::locks::{
    AcquiredLock, LockEncouragement, LockId, ScheduleLock, current_scheduler_locks,
};
use alloc::{
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    sync::{Arc, Weak},
    vec::Vec,
};
use boolvec::BoolVec;
use lldebug::{log, logln};
use mem::{
    page::PhysPage,
    paging::VmPermissions,
    virt2phys::virt2phys,
    vm::{VmProcess, VmRegion},
};

/// A priority queue item with a weak reference to its owned thread
#[derive(Debug)]
struct ScheduleItem {
    priority: isize,
    thread: WeakThread,
}

impl Eq for ScheduleItem {}
impl PartialEq for ScheduleItem {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl PartialOrd for ScheduleItem {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.priority.cmp(&other.priority))
    }
}

impl Ord for ScheduleItem {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

type NoDropAquiredLockId = usize;
type NoDropLockId = usize;

#[derive(Debug)]
struct LockIdInfo {
    shared_locks: usize,
    exclusive_locks: usize,
    lock_map: BTreeMap<NoDropAquiredLockId, (bool, WeakThread)>,
}

#[derive(Debug)]
pub struct LockHoldings {
    lock_id_alloc: BoolVec,
    aquired_id_alloc: BoolVec,
    aquired_map: BTreeMap<NoDropAquiredLockId, NoDropLockId>,
    id_map: BTreeMap<NoDropLockId, LockIdInfo>,
}

#[derive(Debug, Clone, Copy)]
pub enum LockError {
    WillBlock,
    Deadlock,
}

impl Display for LockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl LockHoldings {
    const fn new() -> Self {
        Self {
            lock_id_alloc: BoolVec::new(),
            aquired_id_alloc: BoolVec::new(),
            aquired_map: BTreeMap::new(),
            id_map: BTreeMap::new(),
        }
    }

    /// Check if this lock has any outstanding locks being held
    pub fn any_outstanding_locks(&self, lock_id: &LockId) -> bool {
        self.id_map
            .get(&lock_id.0)
            .is_some_and(|info| info.shared_locks > 0 || info.exclusive_locks > 0)
    }

    /// Get the number of active shared locks
    pub fn current_shared_locks_for(&self, lock_id: &LockId) -> usize {
        self.id_map
            .get(&lock_id.0)
            .map(|info| info.shared_locks)
            .unwrap_or(0)
    }

    /// Get the number of active exclusive locks
    pub fn current_exclusive_locks_for(&self, lock_id: &LockId) -> usize {
        self.id_map
            .get(&lock_id.0)
            .map(|info| info.exclusive_locks)
            .unwrap_or(0)
    }

    /// Create a new lock id
    pub fn alloc_lock_id(&mut self) -> LockId {
        let lock_id = self.lock_id_alloc.find_first_of(false).unwrap();
        self.lock_id_alloc.set(lock_id, true);
        self.id_map.insert(
            lock_id,
            LockIdInfo {
                shared_locks: 0,
                exclusive_locks: 0,
                lock_map: BTreeMap::new(),
            },
        );

        LockId(lock_id)
    }

    /// Dealloc a lock id
    pub fn dealloc_lock_id(&mut self, lock_id: &LockId) {
        if self.any_outstanding_locks(lock_id) {
            panic!(
                "Tried to dealloc a lock (id={}) with outstanding locks!",
                lock_id.0
            );
        }

        assert!(
            self.lock_id_alloc.get(lock_id.0),
            "Lock was not marked in allocation map"
        );
        self.lock_id_alloc.set(lock_id.0, false);
        self.id_map.remove(&lock_id.0);
    }

    pub fn try_lock_exclusive(
        &mut self,
        current_thread: WeakThread,
        lock_id: &LockId,
    ) -> Result<AcquiredLock, LockError> {
        let lock_id_info = self
            .id_map
            .get_mut(&lock_id.0)
            .expect("Tried to aquire a lock for a lockid that doesn't exist!");

        // If this won't block
        if lock_id_info.shared_locks == 0 && lock_id_info.exclusive_locks == 0 {
            lock_id_info.exclusive_locks += 1;

            let new_aquired_lock_id = self.aquired_id_alloc.find_first_of(false).unwrap();
            self.aquired_id_alloc.set(new_aquired_lock_id, true);
            self.aquired_map.insert(new_aquired_lock_id, lock_id.0);
            lock_id_info
                .lock_map
                .insert(new_aquired_lock_id, (true, current_thread));

            return Ok(AcquiredLock(new_aquired_lock_id));
        }

        // If this will block, check if its because of a deadlock
        if lock_id_info
            .lock_map
            .values()
            .map(|(_, weak_thread)| weak_thread)
            .any(|weak_thread| weak_thread.ptr_eq(&current_thread))
        {
            Err(LockError::Deadlock)
        } else {
            Err(LockError::WillBlock)
        }
    }

    pub fn try_lock_shared(
        &mut self,
        current_thread: WeakThread,
        lock_id: &LockId,
    ) -> Result<AcquiredLock, LockError> {
        let lock_id_info = self
            .id_map
            .get_mut(&lock_id.0)
            .expect("Tried to aquire a lock for a lockid that doesn't exist!");

        // If this won't block
        if lock_id_info.exclusive_locks == 0 {
            lock_id_info.shared_locks += 1;

            let new_aquired_lock_id = self.aquired_id_alloc.find_first_of(false).unwrap();
            self.aquired_id_alloc.set(new_aquired_lock_id, true);
            self.aquired_map.insert(new_aquired_lock_id, lock_id.0);
            lock_id_info
                .lock_map
                .insert(new_aquired_lock_id, (false, current_thread));

            return Ok(AcquiredLock(new_aquired_lock_id));
        }

        // If this will block, check if its because of a deadlock
        if lock_id_info
            .lock_map
            .values()
            .filter(|(is_exclusive, _)| *is_exclusive)
            .map(|(_, weak_thread)| weak_thread)
            .any(|weak_thread| weak_thread.ptr_eq(&current_thread))
        {
            Err(LockError::Deadlock)
        } else {
            Err(LockError::WillBlock)
        }
    }

    pub fn unlock(&mut self, lock: &AcquiredLock) {
        let lock_id = self
            .aquired_map
            .remove(&lock.0)
            .expect("Attempted to unlock a lock that was never created!");

        let lock_info = self
            .id_map
            .get_mut(&lock_id)
            .expect("Tried to unlock a lock from a lock that doesn't exist");

        // Is this lock an exclusive lock
        if lock_info
            .lock_map
            .remove(&lock.0)
            .expect("Aquired lock not found in parent's lock")
            .0
        {
            lock_info.exclusive_locks = lock_info.exclusive_locks.checked_sub(1).unwrap();
        } else {
            lock_info.shared_locks = lock_info.shared_locks.checked_sub(1).unwrap();
        }

        self.aquired_id_alloc.set(lock.0, false);
    }
}

pub type RefScheduler = Arc<Scheduler>;
pub type WeakScheduler = Weak<Scheduler>;

static THE_SCHEDULER: ScheduleLock<Option<RefScheduler>> = ScheduleLock::new(None);

#[derive(Debug)]
pub struct Scheduler {
    /// Weak references to all processes.
    ///
    /// Each Process contains a strong reference to the scheduler, and the scheduler only needs to know
    /// the processes exist.
    process_list: ScheduleLock<BTreeMap<ProcessId, WeakProcess>>,
    /// All the threads
    thread_list: ScheduleLock<Vec<RefThread>>,
    /// An allocation bitmap of PIDs
    pid_alloc: ScheduleLock<BoolVec>,
    /// Weak references to queued threads
    picking_queue: ScheduleLock<VecDeque<ScheduleItem>>,
    /// The currently running thread
    running: ScheduleLock<Option<RefThread>>,
    /// The currently held locks for processes and threads
    held_locks: ScheduleLock<LockHoldings>,
    /// Kernel Memory Map
    kernel_vm: ScheduleLock<VmProcess>,
}

impl Scheduler {
    /// Get 'THE' scheduler
    ///
    /// If the scheduler has not be created, this function will create it.
    pub fn get() -> RefScheduler {
        if let Some(sch) = THE_SCHEDULER.lock().clone() {
            return sch;
        } else {
            let mut guard = THE_SCHEDULER.lock();

            logln!("Scheduler Init...");
            let new_scheduler = Arc::new(Self {
                process_list: ScheduleLock::new(BTreeMap::new()),
                picking_queue: ScheduleLock::new(VecDeque::new()),
                running: ScheduleLock::new(None),
                held_locks: ScheduleLock::new(LockHoldings::new()),
                kernel_vm: ScheduleLock::new(VmProcess::new()),
                pid_alloc: ScheduleLock::new(BoolVec::new()),
                thread_list: ScheduleLock::new(Vec::new()),
            });

            *guard = Some(new_scheduler.clone());
            new_scheduler
        }
    }

    /// Begin mapping core kernel regions
    pub unsafe fn init_kernel_vm(
        &self,
        kernel_exe: VmRegion,
        kernel_heap: VmRegion,
        kernel_stack: VmRegion,
        initfs: VmRegion,
    ) {
        // We want to hold this lock the duration of init the kernel regions
        let kernel_vm = self.kernel_vm.lock();

        let mut mapping_counter = 0;
        let mut map_vm_object = |region: VmRegion, permissions: VmPermissions| {
            let mut kernel_mappings = BTreeMap::new();
            for kernel_vpage in region.pages_iter() {
                mapping_counter += 1;

                kernel_mappings.insert(
                    kernel_vpage,
                    PhysPage::containing_addr(virt2phys(kernel_vpage.addr()).unwrap()),
                );
            }
            kernel_vm
                .manual_inplace_new_vmobject(region, permissions, kernel_mappings)
                .expect("Unable to map kernel exe region");
        };

        // FIXME: We should figure out a better solution for creating VmObjects from the
        // kernel's bootloader.
        log!("Remapping bootloader's regions...");
        map_vm_object(kernel_exe, VmPermissions::SYS_RE);
        map_vm_object(kernel_heap, VmPermissions::SYS_RW);
        map_vm_object(kernel_stack, VmPermissions::SYS_RW);
        map_vm_object(initfs, VmPermissions::SYS_R);
        unsafe { kernel_vm.page_tables.clone().load() }.unwrap();
        logln!("OK ({mapping_counter})");
    }

    /// Clone the `VmProcess` instance of the kernel's memory map
    pub fn fork_kernel_vm(&self) -> VmProcess {
        self.kernel_vm.lock().clone()
    }

    /// Create a new PID
    pub fn alloc_pid(&self) -> ProcessId {
        let mut pid_lock = self.pid_alloc.lock();

        let bit_index = pid_lock
            .find_first_of(false)
            .expect("Unable to allocate new PID");
        pid_lock.set(bit_index, true);

        bit_index
    }

    /// Add a process to the process mapping
    pub fn register_new_process(&self, p: RefProcess) {
        logln!("Spawn Process '{}' (pid='{}')", p.name, p.id);
        if let Some(old_proc) = self.process_list.lock().insert(p.id, Arc::downgrade(&p)) {
            if let Some(old_proc) = old_proc.upgrade() {
                panic!(
                    "Cannot replace an 'alive' process with another alive process. ({} tried to replace {})",
                    p.name, old_proc.name
                );
            }
        }
    }

    /// Register a new thread
    pub fn register_new_thread(&self, t: RefThread) {
        {
            // Strong because locking the thread list will cause the process to fully block
            let mut thread_list = t.process.threads.write(LockEncouragement::Strong);
            thread_list.insert(t.id, Arc::downgrade(&t));
        }

        self.thread_list.lock().push(t.clone());
        self.picking_queue.lock().push_back(ScheduleItem {
            priority: 0,
            thread: Arc::downgrade(&t),
        });
    }

    /// Get the currently running thread
    pub fn current_thread(&self) -> WeakThread {
        match &*self.running.lock() {
            Some(thread) => Arc::downgrade(thread),
            None => WeakThread::new(),
        }
    }

    /// Remove a process from the process mapping
    pub fn remove_process(&self, p: &Process) {
        logln!("Kill Process '{}' (pid='{}')", p.name, p.id);
        let mut pid_lock = self.pid_alloc.lock();
        assert!(
            self.process_list.lock().remove(&p.id).is_some(),
            "Cannot remove process, as process was never registered"
        );
        assert!(
            pid_lock.get(p.id),
            "Process ID was already marked as false!"
        );

        pid_lock.set(p.id, false);
    }

    /// Returns the next process that should execute
    fn next(&self) -> RefThread {
        self.picking_queue
            .lock()
            .pop_front()
            .unwrap()
            .thread
            .upgrade()
            .unwrap()
    }

    /// Yield the current thread (If possible)
    pub fn yield_me() {
        if current_scheduler_locks() != 0 {
            todo!("skipping yield");
        }

        let s = Scheduler::get();
        let mut running_lock = s.running.lock();

        // Save the current running process
        if let Some(previous_running) = running_lock.clone() {
            s.picking_queue.lock().push_back(ScheduleItem {
                priority: 0,
                thread: Arc::downgrade(&previous_running),
            });

            // Pick the next running thread
            let next_running = s.next();
            *running_lock = Some(next_running.clone());

            logln!(
                "Yielding from '{}' (pid={}, tid={}) to '{}' (pid={}, tid={})",
                previous_running.process.name,
                previous_running.process.id,
                previous_running.id,
                next_running.process.name,
                next_running.process.id,
                next_running.id
            );

            let previous_task_ptr = previous_running.task.as_ptr();
            let new_task_ptr = next_running.task.as_ptr();

            drop(running_lock);
            drop(s);

            unsafe { Task::switch_task(previous_task_ptr, new_task_ptr) };
        } else {
            // Pick the next running thread
            let next_running = s.next();
            *running_lock = Some(next_running.clone());

            logln!(
                "Yielding to '{}' (pid={}, tid={})",
                next_running.process.name,
                next_running.process.id,
                next_running.id
            );

            let new_task_ptr = next_running.task.as_ptr();

            drop(running_lock);
            drop(s);

            unsafe { Task::switch_first(new_task_ptr) };
        }
    }

    pub fn alloc_new_lockid(&self) -> LockId {
        self.held_locks.lock().alloc_lock_id()
    }

    pub fn dealloc_lockid(&self, lock: &LockId) {
        self.held_locks.lock().dealloc_lock_id(lock);
    }

    pub fn acquiredlock_exclusive(
        lock_id: &LockId,
        encouragement: LockEncouragement,
    ) -> AcquiredLock {
        loop {
            match Scheduler::get().try_acquiredlock_exclusive(lock_id, encouragement) {
                Ok(lock) => break lock,
                Err(LockError::WillBlock) => Self::yield_me(),
                Err(LockError::Deadlock) => {
                    panic!("Aquiring an exclusive lock on this thread will deadlock!")
                }
            }
        }
    }

    pub fn acquiredlock_shared(lock_id: &LockId, encouragement: LockEncouragement) -> AcquiredLock {
        loop {
            match Scheduler::get().try_acquiredlock_shared(lock_id, encouragement) {
                Ok(lock) => break lock,
                Err(LockError::WillBlock) => Self::yield_me(),
                Err(LockError::Deadlock) => {
                    panic!("Aquiring an exclusive lock on this thread will deadlock!")
                }
            }
        }
    }

    pub fn try_acquiredlock_exclusive(
        &self,
        lock_id: &LockId,
        _encouragement: LockEncouragement,
    ) -> Result<AcquiredLock, LockError> {
        self.held_locks
            .lock()
            .try_lock_exclusive(self.current_thread(), lock_id)
    }

    pub fn try_acquiredlock_shared(
        &self,
        lock_id: &LockId,
        _encouragement: LockEncouragement,
    ) -> Result<AcquiredLock, LockError> {
        self.held_locks
            .lock()
            .try_lock_shared(self.current_thread(), lock_id)
    }

    pub fn lockid_total_shared(&self, lock: &LockId) -> usize {
        self.held_locks.lock().current_shared_locks_for(lock)
    }

    pub fn lockid_total_exclusive(&self, lock: &LockId) -> usize {
        self.held_locks.lock().current_exclusive_locks_for(lock)
    }

    pub fn aquiredlock_unlock(&self, lock: &AcquiredLock) {
        self.held_locks.lock().unlock(lock);
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        panic!("Should never drop the scheduler!");
    }
}
