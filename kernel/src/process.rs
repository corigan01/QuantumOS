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

use crate::context::userspace_entry;
use alloc::{
    boxed::Box,
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    string::String,
    sync::{Arc, Weak},
};
use arch::{
    CpuPrivilege, interrupts,
    pic8259::pic_eoi,
    registers::{ProcessContext, Segment},
};
use boolvec::BoolVec;
use core::{
    error::Error,
    sync::atomic::{AtomicBool, Ordering},
};
use elf::{ElfErrorKind, elf_owned::ElfOwned};
use lldebug::{logln, warnln};
use mem::{
    addr::VirtAddr,
    page::VirtPage,
    paging::{PageCorrelationError, PageTableLoadingError, Virt2PhysMapping, VmPermissions},
    vm::{InsertVmObjectError, PageFaultInfo, PageFaultReponse, VmFillAction, VmProcess, VmRegion},
};
use quantum_portal::ExitReason;
use spin::RwLock;
use tar::{Tar, TarError};
use util::consts::{PAGE_2M, PAGE_4K};
use vm_elf::VmElfInject;

pub mod vm_elf;

/// A structure repr a running process on the system
#[derive(Debug, Clone)]
pub struct Process {
    name: String,
    vm: VmProcess,
    id: usize,
    context: ProcessContext,
    cpu_time: u128,
}

#[derive(Debug)]
#[allow(unused)]
pub enum AccessViolationReason {
    /// The user does not have access to this memory
    NoAccess {
        page_perm: VmPermissions,
        request_perm: VmPermissions,
        page: VirtPage,
    },
    /// Other
    Other(Box<dyn Error>),
}

/// A structure repr the errors that could happen with a process
#[derive(Debug)]
#[allow(unused)]
pub enum ProcessError {
    /// There was a problem loading the elf file
    ElfLoadingError(ElfErrorKind),
    /// There was a problem mapping the VmObject
    InsertVmObjectErr(InsertVmObjectError),
    /// There was a problem loading the page tables
    PageTableLoadingErr(PageTableLoadingError),
    /// Process required 'load' assertion error
    ///
    /// This flag tells you if the assertion was to have the table loaded (true)
    /// or unloaded (false).
    LoadedAssertionError(bool),
    /// The process should never return from the enter userspace function
    ProcessShouldNotExit,
    /// No loaded process is currently active, and the requested action depends on a
    /// process context being currently active.
    NotAnyProcess,
    /// An error with loading the initfs files
    InitFsError(TarError),
    /// An error with mapping virtual address regions to physical
    PageCorrelationErr(PageCorrelationError),
    /// There was no such process for PID
    NoSuchProcess(usize),
    /// This process tried to access resources it does not have access to
    AccessViolation(AccessViolationReason),
    /// There was no remaining Virtual Memory to allocate to this process
    OutOfVirtualMemory,
}

impl core::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl core::error::Error for ProcessError {}

impl Process {
    /// This is the start address that processes should base their stack from
    pub const PROCESS_STACK_START_ADDR: VirtAddr = VirtAddr::new(0x7fff00000000);
    /// The stack at which the kernel's IRQ entries should point
    pub const KERNEL_IRQ_STACK_ADDR: VirtAddr = VirtAddr::new(0xffffffff90000000);
    /// The stack at which the kernel's syscall entry should point
    pub const KERNEL_SYSCALL_STACK_ADDR: VirtAddr = VirtAddr::new(0xffffffff92000000);
    /// The size of each of the kernel's stacks
    pub const KERNEL_STACK_SIZE: usize = PAGE_4K * 32;
    /// The size of an userspace process stack
    pub const USERSPACE_STACK_SIZE: usize = PAGE_4K * 16;

    /// Create a new empty process
    pub fn new(id: usize, name: String, table: Virt2PhysMapping) -> Self {
        Self {
            vm: VmProcess::inhearit_page_tables(table),
            id,
            context: ProcessContext::new(),
            cpu_time: 0,
            name,
        }
    }

    /// Get the name of the process
    pub fn get_process_name(&self) -> &str {
        &self.name
    }

    /// Get the Process ID
    pub fn get_pid(&self) -> usize {
        self.id
    }

    /// Loads this processes page tables into memory
    pub unsafe fn load_tables(&self) -> Result<(), ProcessError> {
        // If they are already loaded, we don't need to do anything :)
        if self.vm.page_tables.is_loaded() {
            return Ok(());
        }

        unsafe {
            self.vm
                .page_tables
                .clone()
                .load()
                .map_err(|err| ProcessError::PageTableLoadingErr(err))
        }
    }

    /// Add an elf to process's memory map
    pub fn add_elf(&self, elf: ElfOwned) -> Result<(), ProcessError> {
        let (start, end) = elf.elf().vaddr_range().unwrap();
        let inject_el = VmFillAction::convert(VmElfInject::new(elf));

        self.vm
            .inplace_new_vmobject(
                VmRegion::from_containing(VirtAddr::new(start), VirtAddr::new(end)),
                VmPermissions::none()
                    .set_exec_flag(true)
                    .set_read_flag(true)
                    .set_write_flag(true)
                    .set_user_flag(true),
                inject_el.clone(),
                false,
            )
            .map_err(|err| ProcessError::InsertVmObjectErr(err))?;

        Ok(())
    }

    /// Init the process's context structure
    pub fn init_context(&mut self, entry: VirtAddr, stack: VirtAddr) {
        self.context = ProcessContext::new();
        self.context.ss = Segment::new(4, CpuPrivilege::Ring3).0 as u64;
        self.context.cs = Segment::new(5, CpuPrivilege::Ring3).0 as u64;
        self.context.rip = entry.addr() as u64;
        self.context.rsp = stack.addr() as u64;
        self.context.rflag = 0x200;
    }

    /// Map memory anywhere
    pub fn add_anywhere(
        &self,
        n_pages: usize,
        permissions: VmPermissions,
        populate_now: bool,
    ) -> Result<VmRegion, ProcessError> {
        let region_free = self
            .vm
            .find_vm_free(VirtPage::containing_addr(VirtAddr::new(PAGE_2M)), n_pages)
            .ok_or(ProcessError::OutOfVirtualMemory)?;

        logln!(
            "'{}' is requesting memory! n_pages={}, perm={} (page={})",
            self.name,
            n_pages,
            permissions,
            region_free.start.page()
        );

        self.add_anon(region_free, permissions, populate_now)
            .map(|_| region_free)
    }

    /// Map an anon zeroed scrubbed region to this local process
    pub fn add_anon(
        &self,
        region: VmRegion,
        permissions: VmPermissions,
        populate_now: bool,
    ) -> Result<(), ProcessError> {
        self.vm
            .inplace_new_vmobject(region, permissions, VmFillAction::Scrub(0), populate_now)
            .map_err(|err| ProcessError::InsertVmObjectErr(err))?;

        Ok(())
    }

    /// Make a new process with elf
    pub fn new_exec(
        pid: usize,
        name: String,
        table: Virt2PhysMapping,
        elf: ElfOwned,
    ) -> Result<Self, ProcessError> {
        let mut proc = Self::new(pid, name, table);

        // Point to the entrypoint and the stack
        proc.init_context(
            elf.elf()
                .entry_point()
                .map_err(|err| ProcessError::ElfLoadingError(err))?
                .into(),
            Self::PROCESS_STACK_START_ADDR,
        );

        // Load the elf file
        proc.add_elf(elf)?;

        // Map process's stack
        proc.add_anon(
            VmRegion {
                start: VirtPage::containing_addr(
                    Self::PROCESS_STACK_START_ADDR.sub_offset(Self::USERSPACE_STACK_SIZE),
                ),
                end: VirtPage::containing_addr(Self::PROCESS_STACK_START_ADDR),
            },
            VmPermissions::none()
                .set_exec_flag(true /*FIXME: We need to enable !exec support */)
                .set_read_flag(true)
                .set_write_flag(true)
                .set_user_flag(true),
            false,
        )?;

        Ok(proc)
    }

    /// This process's page fault handler
    pub fn page_fault_handler(&self, info: PageFaultInfo) -> PageFaultReponse {
        self.vm.page_fault_handler(info)
    }

    /// Context switch into this process
    pub unsafe fn switch_into(&self) -> Result<(), ProcessError> {
        logln!("Switching to '{}'...", self.name);
        if !self.vm.page_tables.is_loaded() {
            return Err(ProcessError::LoadedAssertionError(true));
        }

        // Before we switch to the currently running process, we need
        // to release any locks on the scheduler that accumulated
        // when handling this event.
        unsafe { THE_SCHEDULER.force_write_unlock() };
        unsafe { release_scheduler_biglock() };

        unsafe { userspace_entry(&raw const self.context) };
        Err(ProcessError::ProcessShouldNotExit)
    }
}

type WeakRefProcess = Weak<RwLock<Process>>;
type RefProcess = Arc<RwLock<Process>>;
type KernelTicks = usize;

/// The main scheduler object
pub static THE_SCHEDULER: RwLock<Option<Scheduler>> = RwLock::new(None);
pub static IS_SCHEDULER_LOCKED: AtomicBool = AtomicBool::new(false);

/// Force the kernel to lock the scheduler
///
/// # Safety
/// This macro does not check if the scheduler is currently under a lock, and
/// can only be used if the caller is sure the scheduler is not going to be
/// used anywhere else.
///
/// This macro also disables interrupts, and effectively locks the kernel
macro_rules! biglock {
    ($($block:tt)*) => {{
        unsafe { aquire_scheduler_biglock() };
        let biglock_macro_return_value = { $($block)* };
        unsafe { release_scheduler_biglock() };

        biglock_macro_return_value
    }};
}

/// Wait for lock and get a ref to the scheduler object
pub fn lock_ref_scheduler<F, R>(func: F) -> R
where
    F: FnOnce(&Scheduler) -> R,
{
    if let Some(sch) = THE_SCHEDULER.read().as_ref() {
        func(sch)
    } else {
        panic!("The global scheduler has not been setup!")
    }
}

/// Get a mut ref to the scheduler.
///
/// # Note
/// This function asserts that the big lock is already locked.
pub fn lock_mut_scheduler<F, R>(func: F) -> R
where
    F: FnOnce(&mut Scheduler) -> R,
{
    assert_scheduler_biglock(true);
    let mut sch = loop {
        match THE_SCHEDULER.try_write() {
            Some(aquired) => break aquired,
            None => {
                // Relax this kernel thread
                todo!()
            }
        }
    };

    if let Some(sch) = sch.as_mut() {
        func(sch)
    } else {
        panic!("The global scheduler has not been setup!")
    }
}

/// Force the kernel to lock the scheduler
///
/// # Safety
/// Can only be used if the caller is sure the scheduler is not going to be
/// used anywhere else.
///
/// This function also disables interrupts, and effectively locks the kernel
pub unsafe fn aquire_scheduler_biglock() {
    unsafe { interrupts::disable_interrupts() };
    assert_eq!(
        IS_SCHEDULER_LOCKED.swap(true, Ordering::Acquire),
        false,
        "Cannot lock an already locked scheduler biglock"
    );
}

/// Check if the scheduler is currently held in a big lock
pub fn is_scheduler_biglocked() -> bool {
    IS_SCHEDULER_LOCKED.load(Ordering::Relaxed)
}

/// Assert if the scheduler is locked/unlocked
pub fn assert_scheduler_biglock(should_be_locked: bool) {
    assert_eq!(
        IS_SCHEDULER_LOCKED.load(Ordering::Relaxed),
        should_be_locked,
        "Expected the scheduler to be {}, but was {}!",
        if should_be_locked {
            "locked"
        } else {
            "unlocked"
        },
        if should_be_locked {
            "unlocked"
        } else {
            "locked"
        }
    );
}

/// Force the kernel to unlock the scheduler
///
/// # Safety
/// Since this function re-enables interrupts, you must be certain that you do not
/// need the scheduler anymore.
pub unsafe fn release_scheduler_biglock() {
    interrupts::assert_interrupts(false);
    assert_eq!(
        IS_SCHEDULER_LOCKED.swap(false, Ordering::Release),
        true,
        "Cannot release scheduler biglock, when the scheduler is not currently locked!"
    );
    unsafe { interrupts::enable_interrupts() };
}

/// Aquire the currently running process
///
/// This function also releases the scheduler lock.
///
/// # Note
/// This function **must** be called from the process itself.
pub fn aquire_running_and_release_biglock() -> RefProcess {
    let running =
        lock_ref_scheduler(|s| s.running.clone()).expect("Cannot aquire running process!");
    unsafe { release_scheduler_biglock() };

    running
}

/// Set this object as the global scheduler
pub fn set_global_scheduler(sc: Scheduler) -> ! {
    // The biglock gets disabled when we switch back to userspace
    unsafe { aquire_scheduler_biglock() };

    if let Some(old) = THE_SCHEDULER
        .try_write()
        .expect("Should not be holding a read lock to the scheduler if we have not added one yet!")
        .replace(sc)
    {
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
    lock_ref_scheduler(|sc| sc.handle_page_fault(info))
}

/// Send the scheduler an event
pub fn send_scheduler_event(event: SchedulerEvent) {
    match event {
        // Since the tick is low on priority, we don't try to lock
        // if we are already locked!
        SchedulerEvent::Tick(_) => match THE_SCHEDULER.try_read() {
            Some(e) if e.is_some() => {
                drop(e);
                lock_mut_scheduler(|sc| sc.scheduler_event(event));
            }
            _ => (),
        },
        _ => {
            if THE_SCHEDULER.read().is_some() {
                lock_mut_scheduler(|sc| sc.scheduler_event(event));
            }
        }
    }
}

/// Trigger an event to happen on the scheduler
#[derive(Debug)]
#[allow(unused)]
pub enum SchedulerEvent<'a> {
    None,
    /// This is fired by the kernel's timer, and will 'tick' the scheduler forward.
    Tick(&'a ProcessContext),
    /// This process requested to exit
    Exit(ExitReason),
    /// This process encourted a fault and needs to be killed
    Fault(ProcessError),
}

#[derive(Debug)]
pub struct Scheduler {
    /// The kernel's memory mappings
    kernel_table: VmProcess,
    /// A vector of all processes running on the system
    alive: BTreeMap<usize, RefProcess>,
    /// A map of PIDs that are free for processes to use
    pid_bitmap: BoolVec,
    /// A Que of processes needing to be scheduled with the 'KernelTicks'
    /// being the time until that process should be woken up.
    backlog_wake_up: VecDeque<(KernelTicks, WeakRefProcess)>,
    /// The que of processes needing/currently being scheduled.
    ///
    /// The `KernelTicks` here is the remaining time this process has to run before being
    /// send to the end of the que.
    que: VecDeque<(KernelTicks, WeakRefProcess)>,
    /// The process that is activly running
    running: Option<RefProcess>,
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
            que: VecDeque::new(),
            running: None,
            kernel_table: VmProcess::new(),
        }
    }

    /// Get a ref to the currently running process.
    pub fn acquire_running_process(&self) -> Result<RefProcess, ProcessError> {
        self.running.clone().ok_or(ProcessError::NotAnyProcess)
    }

    /// Insert a new process into the schedulers mapping, and get it ready to be scheduled
    pub fn add_process<F>(&mut self, f: F) -> Result<(), ProcessError>
    where
        F: FnOnce(usize, &Virt2PhysMapping) -> Result<Process, ProcessError>,
    {
        let next_pid = self.pid_bitmap.find_first_of(false).unwrap_or(0);
        let proc = Arc::new(RwLock::new(f(next_pid, &self.kernel_table.page_tables)?));

        self.pid_bitmap.set(next_pid, true);

        // We should never override a previous proc
        if let Some(last_proc) = self.alive.insert(next_pid, proc.clone()) {
            panic!(
                "We should never override a previously alive process by inserting a new one. \nOld={:#?}",
                last_proc
            );
        }

        // Mark this process as alive, and insert it into the alive mapping
        self.alive.insert(next_pid, proc.clone());

        // Insert this process into the picking que
        self.que
            .push_back((Self::DEFAULT_QUANTUM, Arc::downgrade(&proc)));

        Ok(())
    }

    /// Setup kernel's memory, like stack, heap, etc...
    fn setup_kernel_memory_regions(&mut self) -> Result<(), ProcessError> {
        // kernel syscall stack
        self.kernel_table
            .inplace_new_vmobject(
                VmRegion {
                    start: VirtPage::containing_addr(
                        Process::KERNEL_SYSCALL_STACK_ADDR.sub_offset(Process::KERNEL_STACK_SIZE),
                    ),
                    end: VirtPage::containing_addr(Process::KERNEL_SYSCALL_STACK_ADDR),
                },
                VmPermissions::none()
                    .set_exec_flag(true)
                    .set_read_flag(true)
                    .set_write_flag(true),
                VmFillAction::Scrub(0),
                true,
            )
            .map_err(|err| ProcessError::InsertVmObjectErr(err))?;

        // kernel irq stack
        self.kernel_table
            .inplace_new_vmobject(
                VmRegion {
                    start: VirtPage::containing_addr(
                        Process::KERNEL_IRQ_STACK_ADDR.sub_offset(Process::KERNEL_STACK_SIZE),
                    ),
                    end: VirtPage::containing_addr(Process::KERNEL_IRQ_STACK_ADDR),
                },
                VmPermissions::none()
                    .set_exec_flag(true)
                    .set_read_flag(true)
                    .set_write_flag(true),
                VmFillAction::Scrub(0),
                true,
            )
            .map_err(|err| ProcessError::InsertVmObjectErr(err))?;

        Ok(())
    }

    /// bootstrap the scheduler from boot
    ///
    /// This is the function called when the kernel is ready to start scheduling processes
    /// and begin mapping its own memory.
    pub fn bootstrap_scheduler(initfs: &'static [u8]) -> Result<Self, ProcessError> {
        let mut sc = Self::new();

        // 1. Copy the bootloader's memory mappings
        sc.kernel_table = VmProcess::inhearit_page_tables(
            unsafe { Virt2PhysMapping::inhearit_bootloader() }
                .map_err(|err| ProcessError::PageCorrelationErr(err))?,
        );

        // 2. Enable those tables
        unsafe { sc.kernel_table.page_tables.clone().load() }
            .map_err(|err| ProcessError::PageTableLoadingErr(err))?;

        // 3. Map memory the kernel will normally use
        sc.setup_kernel_memory_regions()?;

        // 4. load all initfs's elf files
        let tar_file = Tar::new(&initfs);
        tar_file.iter().try_for_each(|tar_file| {
            let filename = tar_file
                .filename()
                .map_err(|tar_err| ProcessError::InitFsError(tar_err))?;

            logln!("loading initfs elf file: '{}'", filename);

            let file = tar_file
                .file()
                .map_err(|tar_err| ProcessError::InitFsError(tar_err))?;

            sc.add_process(|pid, kernel_table| {
                Process::new_exec(
                    pid,
                    filename.into(),
                    Virt2PhysMapping::inhearit_from(kernel_table),
                    ElfOwned::new_from_slice(file),
                )
            })
        })?;

        Ok(sc)
    }

    /// Kill a process with its id
    pub fn kill_proc(&mut self, pid: usize) -> Result<(), ProcessError> {
        // Drop the process if its alive
        if self.alive.remove(&pid).is_some() {
            self.pid_bitmap.set(pid, false);
            Ok(())
        } else {
            Err(ProcessError::NoSuchProcess(pid))
        }
    }

    /// Pick the next process to be enqued
    pub fn pick_next_into_running(&mut self) -> Result<(), ProcessError> {
        // Release running proc
        self.running = None;

        // Get the next process
        let next_head = self.cycle_head()?;
        self.running = Some(next_head);

        Ok(())
    }

    /// Handle events with the scheduler
    pub fn scheduler_event(&mut self, event: SchedulerEvent) {
        match event {
            SchedulerEvent::Exit(_) => {
                let running = self
                    .running
                    .clone()
                    .expect("Expected a currently running process to issue 'Exit'")
                    .read()
                    .clone();

                warnln!("Exiting '{}'", running.name);

                self.kill_proc(running.id)
                    .expect("Expected to be able to kill running process");
                self.pick_next_into_running()
                    .expect("Expected to be able to pick next process");

                self.switch_to_running()
                    .expect("Cannot switch to next running process");

                unreachable!("Should never return from switch to running!")
            }
            SchedulerEvent::Fault(fault) => {
                let running_pid = self
                    .running
                    .clone()
                    .expect("Expected a currently running process to issue 'Fault'")
                    .read()
                    .id;

                // TODO: We should figure out a way to pipe this fualt to userspace
                //       so they can know why a process crashed!
                warnln!("Process fault: {:#?}", fault);
                self.kill_proc(running_pid)
                    .expect("Was not able to kill fault process");
                self.pick_next_into_running()
                    .expect("Expected to be able to pick next process");

                self.switch_to_running()
                    .expect("Cannot switch to next running process");
                unreachable!("Should never return from switch to running!")
            }
            SchedulerEvent::Tick(context) => {
                let Some(_) = self.running else {
                    // If there isn't a running process, there is nothing to tick
                    return;
                };

                // Since there should always be something in the que if there is
                // something currently running, we can just expect.
                let (quantum_remaining, _) = self.que.get_mut(0).expect(
                    "Already verified running, there should always be something in the que!",
                );

                // Its time to switch to a new process!
                if *quantum_remaining == 0 {
                    // Save its context
                    self.running
                        .as_ref()
                        .expect("Expected a currently running process to issue `Tick`")
                        .try_write()
                        .unwrap()
                        .context = *context;

                    self.pick_next_into_running()
                        .expect("Expected to be able to pick next process");

                    // Since this was a timer tick, we also need to send a EOI
                    //
                    // FIXME: we should have the fault handler do this, and call
                    //        the scheduler back up when its done :)
                    unsafe { pic_eoi(0) };

                    self.switch_to_running()
                        .expect("Cannot switch to next running process");

                    unreachable!("Should never return from switch to running!")
                } else {
                    *quantum_remaining -= 1;
                }
            }
            e => todo!("{e:#?}"),
        }
    }

    /// Switch to the running process
    pub fn switch_to_running(&mut self) -> Result<(), ProcessError> {
        let running = self.running.as_ref().ok_or(ProcessError::NotAnyProcess)?;

        unsafe { running.read().load_tables()? };
        unsafe { (&*running.as_mut_ptr()).switch_into() }
    }

    /// Current/Next alive proc to be scheduled
    pub fn head_proc(&mut self) -> Result<RefProcess, ProcessError> {
        loop {
            if let Some(next_proc) = self.que.get(0) {
                if let Some(still_alive) = next_proc.1.upgrade() {
                    return Ok(still_alive);
                }

                // Drop dead refs
                let _ = self.que.pop_front();
            } else {
                // There are no processes left to be scheduled
                return Err(ProcessError::NotAnyProcess);
            }
        }
    }

    /// Cycles the process que until it reaches a currently alive next process
    ///
    /// If the head of the process que is Weak(Dropped) we need to cycle
    /// again until we reach a process that can be scheduled.
    pub fn cycle_head(&mut self) -> Result<RefProcess, ProcessError> {
        loop {
            let Some(next_head) = self.que.pop_front() else {
                // No more processes left in the que
                return Err(ProcessError::NotAnyProcess);
            };

            // If this is still alive, we push it to the end and get the very next process
            if next_head.1.strong_count() >= 1 {
                // Reset its quantum
                self.que.push_back((Self::DEFAULT_QUANTUM, next_head.1));
            } else {
                continue;
            }

            let (_, proc) = self.que.get(0).ok_or(ProcessError::NotAnyProcess)?;
            return Ok(proc
                .upgrade()
                .expect("Just verified we should've had only alive processes"));
        }
    }

    /// Start scheduling
    unsafe fn begin(&mut self) -> ! {
        let head = self
            .head_proc()
            .expect("Expected there to be a process in the schedule que when starting scheduling!");
        self.running.replace(head);

        self.switch_to_running()
            .expect_err("Unable to switch to the running proc");

        unreachable!("Switching to the running executable should never exit!")
    }

    /// Handle an incoming page fault
    pub fn handle_page_fault(&self, info: PageFaultInfo) -> PageFaultReponse {
        let Some(running) = self.running.as_ref() else {
            return PageFaultReponse::CriticalFault(Box::new(ProcessError::NotAnyProcess));
        };

        running.read().page_fault_handler(info)
    }
}
