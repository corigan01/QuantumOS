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
    collections::btree_map::BTreeMap,
    vec::{self, Vec},
};
use elf::{ElfErrorKind, elf_owned::ElfOwned, tables::SegmentKind};
use lldebug::{hexdump::HexPrint, logln};
use mem::{
    addr::VirtAddr,
    page::VirtPage,
    paging::{PageTableLoadingError, Virt2PhysMapping, VmPermissions},
    vm::{
        InsertVmObjectError, PopulationReponse, VmFillAction, VmInjectFillAction, VmProcess,
        VmRegion,
    },
};
use util::consts::PAGE_4K;

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
        // let elf_sections = elf_object.load_segments()?;

        let inject_el = VmFillAction::convert(elf_object);

        // for (region, perms) in elf_sections {
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
        // }
        logln!(
            "{}",
            unsafe { core::slice::from_raw_parts(start as *const u8, end - start) }.hexdump()
        );
        // let elf_object =
        // self.vm.inplace_new_vmobject(region, permissions, fill_action)
        Ok(())
    }

    /// Map an anon zeroed region to this local process
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

/// An elf backing object for a process's memory map
#[derive(Debug)]
pub struct VmElfInject {
    file: ElfOwned,
}

impl VmElfInject {
    /// Create a new VmElfInject
    pub fn new(elf: ElfOwned) -> Self {
        Self { file: elf }
    }

    /// Get an iterator over the different regions that this elf file
    /// needs to load.
    pub fn load_regions(
        &self,
    ) -> Result<impl Iterator<Item = (VirtAddr, VirtAddr, VmPermissions)> + use<'_>, ProcessError>
    {
        Ok(self
            .file
            .elf()
            .program_headers()
            .map_err(|elf_err| ProcessError::ElfLoadingError(elf_err))?
            .iter()
            .filter(|h| h.segment_kind() == SegmentKind::Load)
            .map(|h| {
                let expected_vaddr = VirtAddr::new(h.expected_vaddr() as usize);
                let perms = VmPermissions::none()
                    .set_exec_flag(h.is_executable() || true)
                    .set_read_flag(h.is_readable() || true)
                    .set_write_flag(h.is_writable() || true)
                    .set_user_flag(true);

                (
                    expected_vaddr,
                    expected_vaddr.offset(h.in_mem_size()),
                    perms,
                )
            }))
    }

    /// Get the load segments as aligned segments and permissions fixed
    ///
    /// Since elf exe don't have to have their segments perfectly page aligned
    /// it is possible for two segments to overlap (in a page sense) so we
    /// take the highest of the two and split them
    pub fn load_segments(
        &self,
    ) -> Result<impl IntoIterator<Item = (VmRegion, VmPermissions)> + use<>, ProcessError> {
        // FIXME: This is a realy bad impl of this, we should change this before anyone sees :)
        let mut pages: BTreeMap<VirtPage, VmPermissions> = BTreeMap::new();

        self.load_regions()?
            .map(|(start, end, perm)| {
                let vm_region = VmRegion::from_containing(start, end);
                vm_region.pages_iter().map(move |page| (page, perm))
            })
            .flatten()
            .inspect(|(page, perm)| {
                logln!("LOAD PAGE [{:?}] - {}", page, perm);
            })
            .for_each(|(page, perm)| {
                if let Some(already_existing_page) = pages.get_mut(&page) {
                    *already_existing_page += perm;
                } else {
                    pages.insert(page, perm);
                }
            });

        let mut acc: BTreeMap<VmPermissions, Vec<VirtPage>> = BTreeMap::new();
        for (page, perm) in pages.into_iter() {
            acc.entry(perm)
                .and_modify(|old| old.push(page))
                .or_insert(alloc::vec![page]);
        }

        Ok(acc.into_iter().map(|(perm, pages)| {
            let mut region = VmRegion::new(pages[0], pages[0]);

            for page in pages {
                if page.page() - 1 == region.end.page() {
                    region.end = VirtPage::new(region.end.page() + 1);
                } else if page.page() + 1 == region.start.page() {
                    region.start = VirtPage::new(region.start.page() - 1);
                }
            }

            (region, perm)
        }))
    }

    /// Convert this object into a FillAction
    pub fn fill_action(self) -> VmFillAction {
        VmFillAction::convert(self)
    }
}

impl VmInjectFillAction for VmElfInject {
    fn requests_all_pages_filled(&self, _parent_object: &mem::vm::VmObject) -> bool {
        true
    }

    /// Put data into this page
    fn populate_page(
        &mut self,
        _parent_object: &mem::vm::VmObject,
        _process: &VmProcess,
        _relative_index: usize,
        vpage: mem::page::VirtPage,
        _ppage: mem::page::PhysPage,
    ) -> mem::vm::PopulationReponse {
        let headers = match self.file.elf().program_headers() {
            Ok(header) => header,
            Err(header_err) => return PopulationReponse::InjectError(Box::new(header_err)),
        };

        let vbuffer = unsafe { core::slice::from_raw_parts_mut(vpage.addr().as_mut_ptr(), 4096) };
        vbuffer.fill(0);

        for header in headers.iter().filter(|header| {
            let start_addr = header.expected_vaddr() as usize;
            let end_addr = start_addr + header.in_mem_size();

            (start_addr <= vpage.addr().addr() + 4096 && end_addr >= vpage.addr().addr())
                && header.segment_kind() == SegmentKind::Load
        }) {
            let elf_memory_buffer = match self.file.elf().program_header_slice(&header) {
                Ok(o) => o,
                Err(err) => return mem::vm::PopulationReponse::InjectError(Box::new(err)),
            };

            let buf_start = vpage
                .addr()
                .addr()
                .saturating_sub(header.expected_vaddr() as usize);
            let vbuffer_offset = (header.expected_vaddr() as usize + buf_start) % PAGE_4K;

            let this_page_buffer = &elf_memory_buffer
                [buf_start..(buf_start + (PAGE_4K - vbuffer_offset)).min(elf_memory_buffer.len())];

            logln!(
                "ELF: [{}] {vbuffer_offset:>5}..{:<5} <-- {:>5}..{:<5}   [{:>16x} - {:<16x}]",
                vpage.page(),
                vbuffer_offset + this_page_buffer.len(),
                buf_start,
                (buf_start + (PAGE_4K - vbuffer_offset)).min(elf_memory_buffer.len()),
                header.expected_vaddr(),
                header.expected_vaddr() as usize + header.in_elf_size(),
            );

            vbuffer[vbuffer_offset..vbuffer_offset + this_page_buffer.len()]
                .copy_from_slice(this_page_buffer);
        }
        mem::vm::PopulationReponse::Okay
    }
}
