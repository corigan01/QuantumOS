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

use alloc::{boxed::Box, collections::btree_map::BTreeMap, vec::Vec};
use elf::{elf_owned::ElfOwned, tables::SegmentKind};
use mem::{
    addr::VirtAddr,
    page::VirtPage,
    paging::VmPermissions,
    vm::{PopulationReponse, VmFillAction, VmInjectFillAction, VmProcess, VmRegion},
};
use util::consts::PAGE_4K;

use super::ProcessError;

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
    // Since this is only called to check if we support paging this region, we are always going to say yes
    //
    // FIXME: We should do more checks on this region to ensure that it is correct
    fn page_fault_handler(
        &self,
        _parent_object: &mem::vm::VmObject,
        _process: &VmProcess,
        _info: mem::vm::PageFaultInfo,
    ) -> mem::vm::PageFaultReponse {
        mem::vm::PageFaultReponse::Handled
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

            vbuffer[vbuffer_offset..vbuffer_offset + this_page_buffer.len()]
                .copy_from_slice(this_page_buffer);
        }
        mem::vm::PopulationReponse::Okay
    }
}
