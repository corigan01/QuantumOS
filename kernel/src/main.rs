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

#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]

mod int;
mod panic;
mod timer;
extern crate alloc;

use alloc::{boxed::Box, format};
use bootloader::KernelBootHeader;
use lldebug::{debug_ready, logln, make_debug};
use mem::{
    alloc::{KernelAllocator, provide_init_region},
    pmm::Pmm,
    vmm::{
        KernelRegionKind, VirtPage, VmPermissions, VmRegion, Vmm,
        backing::{PhysicalBacking, VmBacking},
    },
};
use serial::{Serial, baud::SerialBaud};
use timer::kernel_ticks;
use util::bytes::HumanBytes;

#[global_allocator]
static ALLOC: KernelAllocator = KernelAllocator::new();

make_debug! {
    "Serial": Option<Serial> = Serial::probe_first(SerialBaud::Baud115200);
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
extern "C" fn _start(kbh: u64) -> ! {
    main(unsafe { &*(kbh as *const KernelBootHeader) });
    panic!("Main should not return");
}

#[debug_ready]
fn main(kbh: &KernelBootHeader) {
    logln!("Welcome to the Quantum Kernel!");
    logln!(
        "Free Memory : {}",
        HumanBytes::from(kbh.phys_mem_map.bytes_of(mem::phys::PhysMemoryKind::Free))
    );

    int::attach_interrupts();
    int::enable_pic();
    timer::init_timer();

    logln!(
        "Init Heap Region ({})",
        HumanBytes::from(kbh.kernel_init_heap.1)
    );
    provide_init_region(unsafe {
        core::slice::from_raw_parts_mut(kbh.kernel_init_heap.0 as *mut u8, kbh.kernel_init_heap.1)
    });

    logln!("Init PhysMemoryManager");
    let pmm = Pmm::new(kbh.phys_mem_map).unwrap();
    mem::pmm::set_physical_memory_manager(pmm);

    logln!("Init VirtMemoryManager");
    let mut vmm = Vmm::new();
    vmm.init_kernel_process(
        [
            (
                VmRegion {
                    start: VirtPage::containing_page(kbh.kernel_elf.0),
                    end: VirtPage::containing_page(kbh.kernel_elf.0 + kbh.kernel_elf.1 as u64),
                },
                KernelRegionKind::KernelElf,
            ),
            (
                VmRegion {
                    start: VirtPage::containing_page(kbh.kernel_exe.0),
                    end: VirtPage::containing_page(kbh.kernel_exe.0 + kbh.kernel_exe.1 as u64),
                },
                KernelRegionKind::KernelExe,
            ),
            (
                VmRegion {
                    start: VirtPage::containing_page(kbh.kernel_stack.0),
                    end: VirtPage::containing_page(kbh.kernel_stack.0 + kbh.kernel_stack.1 as u64),
                },
                KernelRegionKind::KernelStack,
            ),
            (
                VmRegion {
                    start: VirtPage::containing_page(kbh.kernel_init_heap.0),
                    end: VirtPage::containing_page(
                        kbh.kernel_init_heap.0 + kbh.kernel_init_heap.1 as u64,
                    ),
                },
                KernelRegionKind::KernelHeap,
            ),
        ]
        .into_iter(),
    )
    .unwrap();

    let boxed: Box<dyn VmBacking> = Box::new(PhysicalBacking::new());
    let process = vmm
        .new_process(
            [(
                VmRegion {
                    start: VirtPage(262147),
                    end: VirtPage(262148),
                },
                VmPermissions::READ | VmPermissions::WRITE | VmPermissions::EXEC,
                format!("Example Region"),
                boxed,
            )]
            .into_iter(),
        )
        .unwrap();

    unsafe { process.write().load_page_tables().unwrap() };

    unsafe {
        let ptr = (262147 * 4096) as *mut u8;

        core::ptr::write_volatile(ptr, 10);
    }

    logln!("Wrote to new process's memory area!");
    logln!("Finished in {}ms", kernel_ticks());
    // dump_allocator();
}
