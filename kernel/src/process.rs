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

use alloc::{
    boxed::Box,
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    sync::Arc,
    vec::Vec,
};
use arch::{CpuPrivilege, critcal_section, registers::Segment};
use boolvec::BoolVec;
use elf::{ElfErrorKind, elf_owned::ElfOwned};
use mem::{
    addr::VirtAddr,
    paging::{PageCorrelationError, PageTableLoadingError, Virt2PhysMapping, VmPermissions},
    vm::{InsertVmObjectError, PageFaultInfo, PageFaultReponse, VmFillAction, VmProcess, VmRegion},
};
use spin::RwLock;
use tar::{Tar, TarError};
use util::consts::PAGE_4K;
use vm_elf::VmElfInject;

use crate::context::{ProcessContext, userspace_entry};

pub mod vm_elf;

/// A structure repr a running process on the system
#[derive(Debug, Clone)]
pub struct Process {
    vm: VmProcess,
    id: usize,
    context: ProcessContext,
    cpu_time: u128,
}

/// A structure repr the errors that could happen with a process
#[derive(Debug)]
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
    pub const KERNEL_IRQ_STACK_ADDR: VirtAddr = VirtAddr::new(0xffffffff81000000);
    /// The stack at which the kernel's syscall entry should point
    pub const KERNEL_SYSCALL_STACK_ADDR: VirtAddr = VirtAddr::new(0xffffffff82000000);
    /// The size of each of the kernel's stacks
    pub const KERNEL_STACK_SIZE: usize = PAGE_4K * 16;

    /// Create a new empty process
    pub fn new(id: usize, table: &Virt2PhysMapping) -> Self {
        Self {
            vm: VmProcess::inhearit_page_tables(table),
            id,
            context: ProcessContext::new(),
            cpu_time: 0,
        }
    }

    /// Loads this processes page tables into memory
    pub unsafe fn load_tables(&self) -> Result<(), ProcessError> {
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
        table: &Virt2PhysMapping,
        elf: ElfOwned,
    ) -> Result<Self, ProcessError> {
        todo!()
    }

    /// This process's page fault handler
    pub fn page_fault_handler(&self, info: PageFaultInfo) -> PageFaultReponse {
        self.vm.page_fault_handler(info)
    }

    /// Context switch into this process
    pub unsafe fn switch_into(&self) -> Result<(), ProcessError> {
        if !self.vm.page_tables.is_loaded() {
            return Err(ProcessError::LoadedAssertionError(true));
        }

        unsafe { userspace_entry(&raw const self.context) };
        Err(ProcessError::ProcessShouldNotExit)
    }
}

type RefProcess = Arc<RwLock<Process>>;
type KernelTicks = usize;

/// The main scheduler object
pub static THE_SCHEDULER: RwLock<Option<Scheduler>> = RwLock::new(None);

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

/// Wait for lock and get a ref to the scheduler object
pub fn lock_mut_scheduler<F, R>(func: F) -> R
where
    F: FnOnce(&mut Scheduler) -> R,
{
    // We cannot hold the write lock and also keep interrupts enabled
    // since we could possibly deadlock trying to aquire the lock again
    critcal_section! {
        if let Some(sch) = THE_SCHEDULER.write().as_mut() {
            func(sch)
        } else {
            panic!("The global scheduler has not been setup!")
        }
    }
}

/// Set this object as the global scheduler
pub fn set_global_scheduler(sc: Scheduler) {
    critcal_section! {
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
    }
}

/// The page fault handler for the scheduler
pub fn scheduler_page_fault_handler(info: PageFaultInfo) -> PageFaultReponse {
    lock_ref_scheduler(|sc| sc.handle_page_fault(info))
}

/// Trigger an event to happen on the scheduler
#[derive(Clone, Copy, Debug)]
pub enum SchedulerEvent {
    /// This is fired by the kernel's timer, and will 'tick' the scheduler forward.
    Tick,
}

#[derive(Debug)]
pub struct Scheduler {
    /// The kernel's memory mappings
    kernel_table: Virt2PhysMapping,
    /// A vector of all processes running on the system
    alive: BTreeMap<usize, RefProcess>,
    /// A map of PIDs that are free for processes to use
    pid_bitmap: BoolVec,
    /// A Que of processes needing to be scheduled with the 'KernelTicks'
    /// being the time until that process should be woken up.
    backlog_wake_up: VecDeque<(KernelTicks, RefProcess)>,
    /// The que of processes needing/currently being scheduled.
    ///
    /// The `KernelTicks` here is the remaining time this process has to run before being
    /// send to the end of the que.
    que: VecDeque<(KernelTicks, RefProcess)>,
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
            kernel_table: Virt2PhysMapping::empty(),
        }
    }

    /// Insert a new process into the schedulers mapping, and get it ready to be scheduled
    pub fn add_process<F>(&mut self, f: F) -> Result<(), ProcessError>
    where
        F: FnOnce(usize, &Virt2PhysMapping) -> Result<Process, ProcessError>,
    {
        let next_pid = self.pid_bitmap.find_first_of(false).unwrap_or(0);

        let proc = Arc::new(RwLock::new(f(next_pid, &self.kernel_table)?));
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
        self.que.push_back((Self::DEFAULT_QUANTUM, proc));

        Ok(())
    }

    /// bootstrap the scheduler from boot
    ///
    /// This is the function called when the kernel is ready to start scheduling processes
    /// and begin mapping its own memory.
    pub fn bootstrap_scheduler(initfs: &'static [u8]) -> Result<Self, ProcessError> {
        let mut sc = Self::new();

        // First thing todo is to load the bootloader's memory mappings
        sc.kernel_table = unsafe { Virt2PhysMapping::inhearit_bootloader() }
            .map_err(|err| ProcessError::PageCorrelationErr(err))?;

        // Then enable those tables
        unsafe { sc.kernel_table.clone().load() }
            .map_err(|err| ProcessError::PageTableLoadingErr(err))?;

        let tar_file = Tar::new(&initfs);
        tar_file.iter().try_for_each(|tar_file| {
            let file = tar_file
                .file()
                .map_err(|tar_err| ProcessError::InitFsError(tar_err))?;

            sc.add_process(|pid, kernel_table| {
                Process::new_exec(pid, kernel_table, ElfOwned::new_from_slice(file))
            })
        })?;

        todo!()
    }

    pub fn scheduler_event(&mut self, event: SchedulerEvent) {
        todo!()
    }

    /// Handle an incoming page fault
    pub fn handle_page_fault(&self, info: PageFaultInfo) -> PageFaultReponse {
        let Some(running) = self.running.as_ref() else {
            return PageFaultReponse::CriticalFault(Box::new(ProcessError::NotAnyProcess));
        };

        running.read().page_fault_handler(info)
    }
}
