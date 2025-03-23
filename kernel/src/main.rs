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
#![feature(naked_functions)]

mod context;
mod gdt;
mod int;
mod locks;
mod panic;
mod process;
mod processor;
mod qemu;
mod syscall_handler;
mod timer;
extern crate alloc;

use core::cell::SyncUnsafeCell;

use arch::supports::cpu_vender;
use bootloader::KernelBootHeader;
use lldebug::{debug_ready, logln, make_debug};
use mem::{
    alloc::{KernelAllocator, provide_init_region},
    pmm::Pmm,
    vm::VmRegion,
};
use process::{
    Process,
    scheduler::{Scheduler, init_virt2phys_provider},
    thread::Thread,
};
use serial::{Serial, baud::SerialBaud};
use util::{bytes::HumanBytes, consts::PAGE_4K};

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
    logln!("Running on a(n) '{:?}' processor.", cpu_vender());
    logln!(
        "Init Heap Region ({})",
        HumanBytes::from(kbh.kernel_init_heap.1)
    );

    provide_init_region(unsafe {
        core::slice::from_raw_parts_mut(kbh.kernel_init_heap.0 as *mut u8, kbh.kernel_init_heap.1)
    });

    gdt::init_kernel_gdt();
    unsafe { gdt::load_tss() };
    int::enable_pic();
    int::attach_interrupts();
    int::attach_syscall();
    unsafe { arch::registers::ia32_efer::set_no_execute_flag(true) };

    logln!("Init PhysMemoryManager");
    let pmm = Pmm::new(kbh.phys_mem_map).unwrap();
    let free_pages = pmm.pages_free().unwrap();

    logln!(
        "Unused Physical Pages {} ({})",
        free_pages,
        HumanBytes::from(free_pages * PAGE_4K)
    );
    mem::pmm::set_physical_memory_manager(pmm);

    logln!("Attached virt2phys provider!");
    init_virt2phys_provider();

    let s = Scheduler::get();
    let initfs_region = VmRegion::from_kbh(kbh.initfs_ptr);
    unsafe {
        s.init_kernel_vm(
            VmRegion::from_kbh(kbh.kernel_exe),
            VmRegion::from_kbh(kbh.kernel_init_heap),
            VmRegion::from_kbh(kbh.kernel_stack),
            initfs_region,
        );
    }

    unsafe { (*INITFS_REGION.get()) = initfs_region };

    let kernel_process = Process::new("kernel".into());
    Thread::new_kernel(kernel_process.clone(), init_stage2);
    Thread::new_kernel(kernel_process.clone(), idle);

    // This will start the scheduler for the first time
    Scheduler::yield_now();
}

static INITFS_REGION: SyncUnsafeCell<VmRegion> = SyncUnsafeCell::new(VmRegion::from_kbh((0, 0)));

/// Tasks required after scheduling is setup to be started.
fn init_stage2() {
    logln!("Starting second-stage init!");
    let s = Scheduler::get();
    unsafe { s.spawn_all_initfs(*INITFS_REGION.get()) };
    timer::init_timer();
}

fn idle() {
    loop {
        let s = Scheduler::get();
        if s.threads_alive() <= 1 {
            logln!("All threads exited!");
            qemu::exit_emulator(qemu::QemuExitStatus::Success);
        }
        Scheduler::yield_now();
    }
}
