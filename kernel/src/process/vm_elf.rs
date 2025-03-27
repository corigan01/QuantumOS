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

use alloc::{boxed::Box, sync::Arc};
use elf::{elf_owned::ElfOwned, tables::SegmentKind};
use mem::vm::{PopulationReponse, VmFillAction, VmInjectFillAction, VmProcess};
use util::consts::PAGE_4K;

/// An elf backing object for a process's memory map
#[derive(Debug)]
pub struct VmElfInject {
    file: Arc<ElfOwned>,
}

impl VmElfInject {
    /// Create a new VmElfInject
    pub fn new(elf: Arc<ElfOwned>) -> Self {
        Self { file: elf }
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

            let buf_end = (buf_start + (PAGE_4K - vbuffer_offset)).min(elf_memory_buffer.len());
            if buf_start > buf_end {
                // panic!(
                //     "Elf buffer index out of range! start={buf_start} end={buf_end} size={}\nvpage={} expected_vpage={}\n{:#?}",
                //     elf_memory_buffer.len(),
                //     vpage.addr(),
                //     header.expected_vaddr(),
                //     header
                // );
                continue;
            }

            let this_page_buffer = &elf_memory_buffer[buf_start..buf_end];

            vbuffer[vbuffer_offset..vbuffer_offset + this_page_buffer.len()]
                .copy_from_slice(this_page_buffer);
        }
        mem::vm::PopulationReponse::Okay
    }
}
