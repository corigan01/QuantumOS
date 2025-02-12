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

use super::{
    KernelTicks, ProcessError, RefProcess, RefThread, WeakRefProcess, WeakRefThread, thread::Thread,
};
use crate::{
    process::Process,
    processor::{get_current_process_id, set_current_process_id},
};
use alloc::{
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    string::String,
    sync::Arc,
    vec::Vec,
};
use arch::{CpuPrivilege, locks::InterruptMutex, registers::ProcessContext};
use boolvec::BoolVec;
use core::arch::asm;
use elf::elf_owned::ElfOwned;
use lldebug::logln;
use mem::{
    paging::Virt2PhysMapping,
    vm::{PageFaultInfo, PageFaultReponse},
};
use quantum_portal::{WaitCondition, WaitSignal};
use tar::Tar;

#[derive(Debug)]
pub struct Scheduler {
    kernel_page_tables: Virt2PhysMapping,
    /// A vector of all processes running on the system
    alive: BTreeMap<usize, RefProcess>,
    /// A map of PIDs that are free for processes to use
    pid_bitmap: BoolVec,
    /// A Que of processes needing to be scheduled with the 'KernelTicks'
    /// being the time until that process should be woken up.
    backlog_wake_up: VecDeque<(KernelTicks, WeakRefProcess, WeakRefThread)>,
    /// The que of processes needing/currently being scheduled.
    ///
    /// The `KernelTicks` here is the remaining time this process has to run before being
    /// send to the end of the que.
    queue: VecDeque<(KernelTicks, WeakRefProcess, WeakRefThread)>,
    /// The process that is activly running
    running: Option<(KernelTicks, RefProcess, RefThread)>,
}

impl Scheduler {
    /// The default amount of ticks to swap processes
    const DEFAULT_QUANTUM: usize = 5;

    /// Make a new empty scheduler
    pub const fn new() -> Self {
        Self {
            alive: BTreeMap::new(),
            backlog_wake_up: VecDeque::new(),
            pid_bitmap: BoolVec::new(),
            queue: VecDeque::new(),
            running: None,
            kernel_page_tables: Virt2PhysMapping::empty(),
        }
    }

    /// Allocate a new pid number
    fn alloc_pid(&mut self) -> usize {
        let pid = self.pid_bitmap.find_first_of(false).unwrap_or(0);
        self.pid_bitmap.set(pid, true);

        pid
    }

    /// Add this process to the schedule queue
    pub fn schedule_thread(
        &mut self,
        proc: RefProcess,
        thread: RefThread,
    ) -> Result<(), ProcessError> {
        self.queue.push_back((
            Self::DEFAULT_QUANTUM,
            Arc::downgrade(&proc),
            Arc::downgrade(&thread),
        ));

        Ok(())
    }

    /// Exit a process, or a process's thread
    pub fn exit(
        &mut self,
        proc: RefProcess,
        thread: Option<RefThread>,
    ) -> Result<(), ProcessError> {
        if let Some(thread) = thread {
            proc.lock().exit_thread(thread)
        } else {
            let pid = proc.lock().id;

            if self.alive.remove(&pid).is_none() {
                return Err(ProcessError::NoSuchProcess(pid));
            }

            // If we are the currently running process, we need to remove it
            if let Some((_, proc, _)) = self.running.clone() {
                if proc.lock().get_pid() == pid {
                    self.running = None;
                }
            }

            // free this pid allocation
            self.pid_bitmap.set(pid, false);

            Ok(())
        }
    }

    /// Create a new process
    pub fn exec(
        &mut self,
        name: String,
        bin: Option<ElfOwned>,
    ) -> Result<RefProcess, ProcessError> {
        let pid = self.alloc_pid();
        logln!("Exec new process: name='{}', pid='{}'", name, pid);
        let mut proc = Process::new(pid, name, self.kernel_page_tables.clone());

        // 1. Setup the process's kernel memory regions
        proc.setup_kernel_memory_regions()?;

        // 2. If we have an executable, we need to spawn a new thread
        let ref_thread = if let Some(elf_file) = bin {
            let rip = elf_file.elf().entry_point()?.into();
            let thread_id = proc.new_thread_id();

            proc.add_elf(elf_file)?;
            proc.alloc_thread_stack(thread_id)?;

            let ref_thread = proc.add_thread(
                thread_id,
                rip,
                Process::PROCESS_STACK_START_ADDR,
                CpuPrivilege::Ring3,
            )?;

            Some(ref_thread)
        } else {
            None
        };

        let ref_proc = Arc::new(InterruptMutex::new(proc));

        // 3. Add this process to the list of alive processes
        self.alive.insert(pid, ref_proc.clone());

        // 4. If this process has a thread, lets try and schedule it
        if let Some(thread) = ref_thread {
            self.schedule_thread(ref_proc.clone(), thread)?;
        }

        Ok(ref_proc)
    }

    /// Startup the scheduler
    pub fn bootstrap_scheduler(initfs: &[u8]) -> Result<Self, ProcessError> {
        let mut s = Self::new();

        // 1. Copy the kernel pages
        s.kernel_page_tables = unsafe { Virt2PhysMapping::inhearit_bootloader()? };
        unsafe { s.kernel_page_tables.clone().load() }.unwrap();

        // 2. Look through the initfs and load all the processes
        let initfs_tar = Tar::new(initfs);

        initfs_tar.iter().try_for_each(|tar_file| {
            let elf_file = ElfOwned::new_from_slice(tar_file.file()?);
            let file_name = tar_file.filename()?.into();

            s.exec(file_name, Some(elf_file)).map(|_| ())
        })?;

        Ok(s)
    }

    /// Get the next process on the process queue
    pub fn next(&mut self) -> Result<(KernelTicks, RefProcess, RefThread), ProcessError> {
        loop {
            // If we have a process that can be pulled off the que
            if let Some((ticks, proc, thread)) = self.queue.get(0).cloned() {
                self.queue.pop_front();
                let Some(upgrade_proc) = proc.upgrade() else {
                    logln!("Dead proc in schedule queue!");
                    continue;
                };
                let Some(upgrade_thread) = thread.upgrade() else {
                    logln!("Dead thread in schedule queue!");
                    continue;
                };

                return Ok((ticks, upgrade_proc, upgrade_thread));
            }

            // If we are dead, we cannot continue!
            if self.alive.len() == 0 {
                return Err(ProcessError::NotAnyProcess);
            }

            // If we do not have a process, we go into waiting mode
            logln!("Halting scheduler to wait for more processes");
            unsafe { asm!("hlt") };
        }
    }

    /// Jump into the running process
    pub unsafe fn switch_to_running(&mut self) -> Result<(), ProcessError> {
        let Some((_, process, thread)) = self.running.clone() else {
            return Err(ProcessError::NotAnyProcess);
        };

        let pid = process.lock().id;
        if get_current_process_id() == pid {
            assert!(
                process.lock().is_loaded(),
                "If last process is current, page tables should be loaded!"
            );
        } else {
            set_current_process_id(pid);
            unsafe { process.lock().load_tables()? };
        };

        unsafe { (&mut *process.as_mut_ptr()).switch_into(thread) }
    }

    /// Tick the scheduler
    ///
    /// Returns true when the scheduler changed the currently running process.
    pub fn tick_scheduler(
        &mut self,
        context: Option<&ProcessContext>,
    ) -> Result<bool, ProcessError> {
        let last_running = self.running.as_mut();

        // Extract the last running process
        let Some((ticks_remaining, proc, thread)) = last_running else {
            // Otherwise fill the running process with the next queued process
            self.running = Some(self.next()?);
            return Ok(true);
        };

        // If this process has ticks remaining, we just decrement them
        if *ticks_remaining > 0 {
            *ticks_remaining -= 1;
            return Ok(false);
        }

        // Set its context
        if let Some(context) = context {
            match (context.cs >> 3) as u16 {
                Thread::USERSPACE_CODE_SEGMENT => unsafe {
                    (&mut *thread.as_mut_ptr()).set_userspace_context(*context)
                },
                Thread::KERNEL_CODE_SEGMENT => unsafe {
                    (&mut *thread.as_mut_ptr()).set_kernel_context(*context)
                },
                segment => unreachable!("Unknown code segment {segment}"),
            }
        }

        // Otherwise, move the currently active process to the end of the queue
        self.queue.push_back((
            Self::DEFAULT_QUANTUM,
            Arc::downgrade(&proc),
            Arc::downgrade(&thread),
        ));
        self.running = Some(self.next()?);

        Ok(true)
    }

    /// Begin scheduling
    pub unsafe fn begin(&mut self) -> ! {
        assert!(
            self.running.is_none(),
            "Cannot begin scheduler with an already active process"
        );
        let (ticks, process, thread) = self
            .next()
            .expect("Expected to start a new thread when begining scheduler");

        self.running = Some((ticks, process.clone(), thread.clone()));
        unsafe {
            self.switch_to_running()
                .expect("Expected to be able to switch to the running process")
        };

        unreachable!("Should never return from process!")
    }

    /// Handle a page fault for the currently active process
    pub fn handle_page_fault(&self, info: PageFaultInfo) -> PageFaultReponse {
        let Some((_, ref proc, _)) = self.running else {
            return PageFaultReponse::NotAttachedHandler;
        };

        proc.lock().page_fault_handler(info)
    }
}

static THE_SCHEDULER: InterruptMutex<Option<Scheduler>> = InterruptMutex::new(None);

pub fn ref_scheduler<F, R>(f: F) -> R
where
    F: FnOnce(&Scheduler) -> R,
{
    let lock = THE_SCHEDULER.lock();
    let s_ref = lock.as_ref().expect("Scheduler has not been set!");

    f(s_ref)
}

pub fn mut_scheduler<F, R>(f: F) -> R
where
    F: FnOnce(&mut Scheduler) -> R,
{
    let mut lock = THE_SCHEDULER.lock();
    let s_ref = lock.as_mut().expect("Scheduler has not been set!");

    f(s_ref)
}

pub fn mut_scheduler_no_exit<F>(f: F) -> !
where
    F: FnOnce(&mut Scheduler),
{
    let lock = THE_SCHEDULER.lock();
    // Release lock does not re-enable interrupts!
    let s_ref = unsafe { lock.release_lock() }
        .as_mut()
        .expect("Scheduler has not been set!");

    f(s_ref);
    unreachable!("Should never return from a no-exit function")
}

/// Get the currently active thread and process
pub fn current_process() -> (RefProcess, RefThread) {
    let (_, proc, thread) = ref_scheduler(|s| {
        s.running
            .clone()
            .expect("Expected a currently running process")
    });

    (proc, thread)
}

/// Set this object as the global scheduler
pub fn set_global_scheduler(sc: Scheduler) -> ! {
    if let Some(old) = unsafe { &mut *THE_SCHEDULER.as_mut_ptr() }.replace(sc) {
        panic!(
            "Attempted to override the global scheduler object, maybe kernel bug? \n{:#?}",
            old
        );
    };

    // Also make sure to attach the page fault handler here too
    mem::vm::set_page_fault_handler(scheduler_page_fault_handler);

    // Now tell the scheduler to start picking a process to use
    //
    // This is only safe because we know we are the only ones who
    // have access to the scheduler since we just placed it here.
    unsafe { (&mut *THE_SCHEDULER.as_mut_ptr()).as_mut().unwrap().begin() };
}

/// The page fault handler for the scheduler
pub fn scheduler_page_fault_handler(info: PageFaultInfo) -> PageFaultReponse {
    ref_scheduler(|sc| sc.handle_page_fault(info))
}

/// Exit this process
pub fn scheduler_exit_process(process: RefProcess) -> ! {
    mut_scheduler_no_exit(|s| {
        s.exit(process, None).expect("expected to exit process");
        assert!(
            s.tick_scheduler(None).expect("Expected to tick scheduler"),
            "There should always be a switch of process after an exit"
        );
        unsafe { s.switch_to_running() }.expect("Could not switch to next proc")
    });
}

/// Tick the scheduler
pub fn scheduler_tick(context: &ProcessContext) -> Result<(), ProcessError> {
    // If we changed our running process, we need to switch to it
    if mut_scheduler(|s| s.tick_scheduler(Some(context)))? {
        mut_scheduler_no_exit(|s| unsafe { s.switch_to_running() }.unwrap());
    }

    Ok(())
}

/// Add thread waiting events
pub fn scheduler_thread_wait(
    waiting_conds: &[WaitCondition],
) -> Result<Vec<WaitSignal>, ProcessError> {
    todo!()
}
