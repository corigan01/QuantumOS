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

use elf::{ElfErrorKind, elf_owned::ElfOwned};
use mem::{
    addr::VirtAddr,
    paging::{PageTableLoadingError, Virt2PhysMapping, VmPermissions},
    vm::{InsertVmObjectError, VmFillAction, VmProcess, VmRegion},
};
use vm_elf::VmElfInject;

pub mod vm_elf;

/// A structure repr a running process on the system
#[derive(Debug)]
pub struct Process {
    vm: VmProcess,
    id: usize,
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
}

impl Process {
    /// Create a new empty process
    pub fn new(id: usize, table: &Virt2PhysMapping) -> Self {
        Self {
            vm: VmProcess::inhearit_page_tables(table),
            id,
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

    /// Asserts that this process should be the currently loaded process to do
    /// the requested action. For example, when populating this processes pages
    /// it needs to be activly loaded.
    pub fn assert_loaded(&self, should_be_loaded: bool) -> Result<(), ProcessError> {
        if self.vm.page_tables.is_loaded() == should_be_loaded {
            Ok(())
        } else {
            Err(ProcessError::LoadedAssertionError(should_be_loaded))
        }
    }

    /// Add an elf to process's memory map
    pub fn add_elf(&self, elf: ElfOwned) -> Result<(), ProcessError> {
        // Page tables need to be loaded before we can map the elf into memory
        self.assert_loaded(true)?;

        let (start, end) = elf.elf().vaddr_range().unwrap();
        let elf_object = VmElfInject::new(elf);
        let inject_el = VmFillAction::convert(elf_object);

        self.vm
            .inplace_new_vmobject(
                VmRegion::from_containing(VirtAddr::new(start), VirtAddr::new(end)),
                VmPermissions::none()
                    .set_exec_flag(true)
                    .set_read_flag(true)
                    .set_write_flag(true)
                    .set_user_flag(true),
                inject_el.clone(),
            )
            .map_err(|err| ProcessError::InsertVmObjectErr(err))?;

        Ok(())
    }

    /// Map an anon zeroed scrubbed region to this local process
    pub fn add_anon(
        &self,
        region: VmRegion,
        permissions: VmPermissions,
    ) -> Result<(), ProcessError> {
        // Page tables need to be loaded before we can map the elf into memory
        self.assert_loaded(true)?;

        self.vm
            .inplace_new_vmobject(region, permissions, VmFillAction::Scrub(0))
            .map_err(|err| ProcessError::InsertVmObjectErr(err))?;

        Ok(())
    }
}
