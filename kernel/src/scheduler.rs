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

use core::alloc::Layout;

use alloc::{alloc::alloc_zeroed, boxed::Box, format, sync::Arc, vec::Vec};
use elf::{elf_owned::ElfOwned, tables::SegmentKind};
use lldebug::logln;
use mem::{
    pmm::{PhysPage, use_pmm_mut},
    vmm::{
        VirtPage, VmPermissions, VmProcess, VmRegion,
        backing::{PhysicalBacking, VmBacking, VmBackingKind},
        use_vmm_mut,
    },
};
use spin::RwLock;
use util::consts::PAGE_4K;

pub struct Process {
    id: usize,
    loc: ElfOwned,
    // TODO: We need to have a VmProcess struct that contains the locking logic inside
    process: Arc<RwLock<VmProcess>>,
}

pub struct Scheduler {
    process: Process,
}

const EXPECTED_START_ADDR: usize = 0x00400000;
const EXPECTED_STACK_ADDR: usize = 0x10000000;
const EXPECTED_STACK_LEN: usize = 4096;

impl Scheduler {
    pub fn new_initfs(initfs: &[u8]) -> Self {
        let elf_owned = ElfOwned::new_from_slice(initfs);
        let elf_regions = elf_owned
            .elf()
            .program_headers()
            .unwrap()
            .iter()
            .filter(|h| h.segment_kind() == SegmentKind::Load)
            .map(|h| {
                let vm_region = VmRegion::from_vaddr(h.expected_vaddr(), h.in_mem_size());
                let binary_backed: Box<dyn VmBacking> = Box::new(BinaryBacked::new(
                    vm_region,
                    elf_owned.elf().program_header_slice(&h).unwrap().into(),
                ));

                (
                    vm_region,
                    VmPermissions::USER | VmPermissions::EXEC | VmPermissions::READ,
                    format!("LOAD"),
                    binary_backed,
                )
            });

        // let exe_backing: Box<dyn VmBacking> = Box::new(ElfBacking::new(exe_region, elf_owned));
        let stack_backing: Box<dyn VmBacking> = Box::new(PhysicalBacking::new());

        let process = use_vmm_mut(|vmm| {
            vmm.new_process(
                [(
                    VmRegion {
                        start: VirtPage(EXPECTED_STACK_ADDR / PAGE_4K),
                        end: VirtPage((EXPECTED_STACK_ADDR + EXPECTED_STACK_LEN) / PAGE_4K + 1),
                    },
                    VmPermissions::USER | VmPermissions::WRITE | VmPermissions::READ,
                    format!("init stack"),
                    stack_backing,
                )]
                .into_iter()
                .chain(elf_regions),
            )
        })
        .unwrap();

        unsafe { process.write().load_page_tables().unwrap() };
        logln!("INIT Page tables loaded!");

        todo!()
    }
}

pub struct BinaryBacked {
    vm_region: VmRegion,
    binary: Vec<u8>,
    pages: Vec<PhysPage>,
}

impl BinaryBacked {
    pub fn new(vm_region: VmRegion, binary: Vec<u8>) -> Self {
        Self {
            vm_region,
            binary,
            pages: Vec::new(),
        }
    }
}

impl VmBacking for BinaryBacked {
    fn backing_kind(&self) -> VmBackingKind {
        VmBackingKind::ElfBackedPhysical
    }

    fn alive_physical_pages(&self) -> Result<Vec<PhysPage>, mem::MemoryError> {
        Ok(self.pages.clone())
    }

    fn alloc_anywhere(&mut self, vpage: VirtPage) -> Result<PhysPage, mem::MemoryError> {
        let phys_page = use_pmm_mut(|pmm| pmm.allocate_page());
        let inner_offset = (vpage - self.vm_region.start).0 * PAGE_4K;

        todo!()
    }

    fn dealloc(&mut self, page: PhysPage) -> Result<(), mem::MemoryError> {
        todo!()
    }
}
