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

use alloc::vec::Vec;
use alloc::{
    boxed::Box,
    string::String,
    sync::{Arc, Weak},
};
use arch::CpuPrivilege;
use arch::interrupts::disable_interrupts;
use arch::locks::InterruptMutex;
use boolvec::BoolVec;
use core::error::Error;
use elf::{ElfErrorKind, elf_owned::ElfOwned};
use lldebug::logln;
use mem::{
    addr::VirtAddr,
    page::VirtPage,
    paging::{PageCorrelationError, PageTableLoadingError, Virt2PhysMapping, VmPermissions},
    vm::{InsertVmObjectError, PageFaultInfo, PageFaultReponse, VmFillAction, VmProcess, VmRegion},
};
use tar::TarError;
use thread::Thread;
use util::consts::{PAGE_2M, PAGE_4K};
use vm_elf::VmElfInject;

use crate::processor::{notify_begin_critical, set_current_process_id};

pub mod scheduler;
pub mod thread;
pub mod vm_elf;

type RefThread = Arc<InterruptMutex<Thread>>;
type WeakRefThread = Weak<InterruptMutex<Thread>>;

/// A structure repr a running process on the system
#[derive(Debug, Clone)]
pub struct Process {
    name: String,
    vm: VmProcess,
    id: usize,
    threads: Vec<RefThread>,
    thread_ids: BoolVec,
}

#[derive(Debug)]
#[allow(unused)]
pub enum AccessViolationReason {
    /// The user does not have access to this memory
    NoAccess {
        page_perm: VmPermissions,
        request_perm: VmPermissions,
        addr: VirtAddr,
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
    /// There is currently no active thread
    NoActiveThreads,
    /// An error with loading the initfs files
    InitFsError(TarError),
    /// An error with mapping virtual address regions to physical
    PageCorrelationErr(PageCorrelationError),
    /// There was no such process for PID
    NoSuchProcess(usize),
    /// There was no such thread for TID
    NoSuchThread(usize),
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

impl Error for ProcessError {}

impl From<ElfErrorKind> for ProcessError {
    fn from(value: ElfErrorKind) -> Self {
        ProcessError::ElfLoadingError(value)
    }
}

impl From<InsertVmObjectError> for ProcessError {
    fn from(value: InsertVmObjectError) -> Self {
        Self::InsertVmObjectErr(value)
    }
}

impl From<PageCorrelationError> for ProcessError {
    fn from(value: PageCorrelationError) -> Self {
        Self::PageCorrelationErr(value)
    }
}

impl From<TarError> for ProcessError {
    fn from(value: TarError) -> Self {
        Self::InitFsError(value)
    }
}

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
            name,
            threads: Vec::new(),
            thread_ids: BoolVec::new(),
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
        if self.is_loaded() {
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

    /// Exit a thread
    pub fn exit_thread(&mut self, thread: RefThread) -> Result<(), ProcessError> {
        todo!()
    }

    /// Checks if the current page tables are loaded
    pub fn is_loaded(&self) -> bool {
        self.vm.page_tables.is_loaded()
    }

    /// Allocate a thread stack for a given thread id
    pub fn alloc_thread_stack(&mut self, thread_id: usize) -> Result<VmRegion, ProcessError> {
        let stack_top = Self::PROCESS_STACK_START_ADDR;
        let stack_inc = Self::USERSPACE_STACK_SIZE;
        let guard_size = PAGE_4K;

        let stack_top_for_thread = stack_top.sub_offset((stack_inc + guard_size) * thread_id);
        let stack_bottom_for_thread = stack_top_for_thread.sub_offset(stack_inc);

        let stack_region = VmRegion::from_containing(stack_bottom_for_thread, stack_top_for_thread);
        self.add_anon(stack_region, VmPermissions::USER_RW, false)?;

        Ok(stack_region)
    }

    /// Setup kernel's memory, like stack, heap, etc...
    fn setup_kernel_memory_regions(&self) -> Result<(), ProcessError> {
        // kernel syscall stack
        self.vm
            .inplace_new_vmobject(
                VmRegion {
                    start: VirtPage::containing_addr(
                        Self::KERNEL_SYSCALL_STACK_ADDR.sub_offset(Self::KERNEL_STACK_SIZE),
                    ),
                    end: VirtPage::containing_addr(Self::KERNEL_SYSCALL_STACK_ADDR),
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
        self.vm
            .inplace_new_vmobject(
                VmRegion {
                    start: VirtPage::containing_addr(
                        Self::KERNEL_IRQ_STACK_ADDR.sub_offset(Self::KERNEL_STACK_SIZE),
                    ),
                    end: VirtPage::containing_addr(Self::KERNEL_IRQ_STACK_ADDR),
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

    /// Check that a virtual address is valid
    pub fn check_addr(
        &self,
        addr: VirtAddr,
        expected_perms: VmPermissions,
    ) -> Result<(), ProcessError> {
        match self.vm.check_addr_perms(addr, expected_perms) {
            mem::vm::CheckAddrResult::MappedAndValidPerms => Ok(()),
            mem::vm::CheckAddrResult::NotMapped => Err(ProcessError::AccessViolation(
                AccessViolationReason::NoAccess {
                    page_perm: VmPermissions::none(),
                    request_perm: expected_perms,
                    addr,
                },
            )),
            mem::vm::CheckAddrResult::MappedInvalidPerms { expected, found } => Err(
                ProcessError::AccessViolation(AccessViolationReason::NoAccess {
                    page_perm: found,
                    request_perm: expected,
                    addr,
                }),
            ),
        }
    }

    /// Allocate a new thread id
    pub fn new_thread_id(&mut self) -> usize {
        let thread_id = self.thread_ids.find_first_of(false).unwrap_or(0);
        self.thread_ids.set(thread_id, true);

        thread_id
    }

    /// Spawn the main thread for this process
    pub fn add_thread(
        &mut self,
        thread_id: usize,
        rip: VirtAddr,
        rsp: VirtAddr,
        ring: CpuPrivilege,
    ) -> Result<RefThread, ProcessError> {
        let is_ring3 = match ring {
            CpuPrivilege::Ring0 => false,
            CpuPrivilege::Ring3 => true,
            _ => panic!("Rings other than Ring0 and Ring3 are not supported"),
        };

        self.check_addr(
            rip,
            VmPermissions::none()
                .set_read_flag(true)
                .set_exec_flag(true)
                // FIXME
                .set_write_flag(true)
                .set_user_flag(is_ring3),
        )?;
        self.check_addr(
            rsp,
            VmPermissions::none()
                .set_write_flag(true)
                .set_read_flag(true)
                .set_user_flag(is_ring3),
        )?;

        let mut new_thread = Thread::new(thread_id);
        if is_ring3 {
            new_thread.init_user_context(rip, rsp);
        } else {
            todo!()
        }

        let ref_thrad = Arc::new(InterruptMutex::new(new_thread));
        self.threads.push(ref_thrad.clone());

        Ok(ref_thrad)
    }

    /// This process's page fault handler
    pub fn page_fault_handler(&self, info: PageFaultInfo) -> PageFaultReponse {
        self.vm.page_fault_handler(info)
    }

    /// Context switch into this process
    pub unsafe fn switch_into(&mut self, thread: RefThread) -> Result<(), ProcessError> {
        logln!(
            "Switching to '{}' (pid={}, tid={})...",
            self.name,
            self.id,
            thread.lock().get_tid()
        );

        // Begin a critical section
        unsafe { disable_interrupts() };
        notify_begin_critical();

        // Set the global process ID
        set_current_process_id(self.id);

        // Page tables must be loaded to switch into the process
        if !self.vm.page_tables.is_loaded() {
            return Err(ProcessError::LoadedAssertionError(true));
        }

        unsafe { (&mut *thread.as_mut_ptr()).context_switch() };
    }
}

type WeakRefProcess = Weak<InterruptMutex<Process>>;
type RefProcess = Arc<InterruptMutex<Process>>;
type KernelTicks = usize;
