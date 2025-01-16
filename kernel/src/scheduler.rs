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

use alloc::{boxed::Box, vec::Vec};
use elf::{elf_owned::ElfOwned, tables::SegmentKind};
use lldebug::{hexdump::HexPrint, logln};
use mem::{
    MemoryError,
    pmm::PhysPage,
    vmm::{
        VirtPage, VmPermissions, VmProcess, VmRegion,
        backing::{KernelVmObject, VmRegionObject},
    },
};
use util::consts::{PAGE_2M, PAGE_4K};

pub struct Process {
    id: usize,
    vm: VmProcess,
}

pub struct Scheduler {
    kernel: Process,
    process_list: Vec<Process>,
}

const EXPECTED_START_ADDR: usize = 0x00400000;
const EXPECTED_STACK_ADDR: usize = 0x10000000;
const EXPECTED_STACK_LEN: usize = 4096;

const KERNEL_HANDLER_RSP: usize = 0x200000000000;

impl Scheduler {
    pub fn new(kernel_process: VmProcess) -> Self {
        Self {
            kernel: Process {
                id: 0,
                vm: kernel_process,
            },
            process_list: Vec::new(),
        }
    }

    pub fn add_initfs(&mut self, initfs: &[u8]) -> Result<(), MemoryError> {
        let elf_owned = ElfOwned::new_from_slice(initfs);

        // Kernel Process Stack
        self.kernel.vm.add_vm_object(KernelVmObject::new_boxed(
            VmRegion {
                start: VirtPage::containing_page((KERNEL_HANDLER_RSP - PAGE_2M) as u64),
                end: VirtPage::containing_page((KERNEL_HANDLER_RSP + PAGE_4K) as u64),
            },
            VmPermissions::READ | VmPermissions::WRITE | VmPermissions::USER | VmPermissions::EXEC,
            false,
        ));
        self.kernel.vm.map_all_now();

        let (vaddr_low, vaddr_hi) = elf_owned
            .elf()
            .vaddr_range()
            .map_err(|_| MemoryError::NotSupported)?;

        let process = VmProcess::new_from(&self.kernel.vm);

        process.add_vm_object(ElfBacked::new_boxed(
            VmRegion::from_vaddr(vaddr_low as u64, vaddr_hi - vaddr_low),
            VmPermissions::WRITE | VmPermissions::READ | VmPermissions::USER | VmPermissions::EXEC,
            elf_owned,
        ));
        process.add_vm_object(NothingBacked::new_boxed(
            VmRegion {
                start: VirtPage(2),
                end: VirtPage(3),
            },
            VmPermissions::WRITE | VmPermissions::READ | VmPermissions::USER,
        ));

        process.map_all_now()?;
        logln!("{:#?}", process);
        logln!(
            "Initfs loading to : V{:#016x}\n{}",
            vaddr_low,
            unsafe { core::slice::from_raw_parts(vaddr_low as *const u8, 128) }.hexdump()
        );
        process.dump_page_tables();

        Ok(())
    }
}

#[derive(Debug)]
pub struct ElfBacked {
    region: VmRegion,
    permissions: VmPermissions,
    // TODO: Make this global and ref to it instead of copying it a bunch of times
    elf: ElfOwned,
}

impl ElfBacked {
    pub fn new_boxed(
        region: VmRegion,
        permissions: VmPermissions,
        elf: ElfOwned,
    ) -> Box<dyn VmRegionObject> {
        Box::new(Self {
            region,
            permissions,
            elf,
        })
    }
}

impl VmRegionObject for ElfBacked {
    fn vm_region(&self) -> VmRegion {
        self.region
    }

    fn vm_permissions(&self) -> VmPermissions {
        self.permissions
    }

    fn init_page(&mut self, vpage: VirtPage, _ppage: PhysPage) -> Result<(), MemoryError> {
        let elf_headers = self
            .elf
            .elf()
            .program_headers()
            .map_err(|_| MemoryError::DidNotHandleException)?
            .iter()
            .filter(|h| h.segment_kind() == SegmentKind::Load)
            .filter(|h| self.region.contains_vaddr(h.expected_vaddr()));

        for header in elf_headers {
            let elf_buffer = self
                .elf
                .elf()
                .program_header_slice(&header)
                .map_err(|_| MemoryError::DidNotHandleException)?;

            let page_offset = header.expected_vaddr() as usize % PAGE_4K;

            let virtal_slice = unsafe {
                core::slice::from_raw_parts_mut(
                    (vpage.0 * PAGE_4K + page_offset) as *mut u8,
                    4096 - page_offset,
                )
            };
            let buffer_offset = (vpage - self.region.start).0 * PAGE_4K;

            // This page is within the elf file
            if elf_buffer.len() >= buffer_offset {
                let elf_buf = &elf_buffer[buffer_offset..];

                virtal_slice[..elf_buf.len()].copy_from_slice(elf_buf);
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct NothingBacked {
    region: VmRegion,
    permissions: VmPermissions,
}

impl NothingBacked {
    pub fn new_boxed(region: VmRegion, permissions: VmPermissions) -> Box<dyn VmRegionObject> {
        Box::new(Self {
            region,
            permissions,
        })
    }
}

impl VmRegionObject for NothingBacked {
    fn vm_region(&self) -> VmRegion {
        self.region
    }

    fn vm_permissions(&self) -> VmPermissions {
        self.permissions
    }
}
