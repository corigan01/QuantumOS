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
mod panic;
mod scheduler;
mod timer;
extern crate alloc;

use arch::CpuPrivilege::Ring0;
use bootloader::KernelBootHeader;
use context::{KERNEL_RSP_PTR, ProcessContext, USERSPACE_RSP_PTR, context_of_caller};
use elf::elf_owned::ElfOwned;
use lldebug::{debug_ready, logln, make_debug};
use mem::{
    addr::VirtAddr,
    alloc::{KernelAllocator, provide_init_region},
    paging::{Virt2PhysMapping, VmPermissions, init_virt2phys_provider},
    pmm::Pmm,
    vm::VmRegion,
};
use scheduler::Process;
use serial::{Serial, baud::SerialBaud};
use tar::Tar;
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
    unsafe { *(KERNEL_RSP_PTR as *mut u64) = 0x200000000000 };
    unsafe { *(USERSPACE_RSP_PTR as *mut u64) = 0 };

    logln!("Welcome to the Quantum Kernel!");
    logln!(
        "Free Memory : {}",
        HumanBytes::from(kbh.phys_mem_map.bytes_of(mem::phys::PhysMemoryKind::Free))
    );

    gdt::init_kernel_gdt();
    gdt::set_stack_for_privl(0x300000000000 as *mut u8, Ring0);
    unsafe { gdt::load_tss() };
    int::attach_interrupts();
    int::attach_syscall();
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
    let free_pages = pmm.pages_free().unwrap();

    logln!(
        "Unused Physical Pages {} ({})",
        free_pages,
        HumanBytes::from(free_pages * PAGE_4K)
    );
    mem::pmm::set_physical_memory_manager(pmm);

    logln!("Attached virt2phys provider!");
    init_virt2phys_provider();

    logln!("Init VirtMemoryManager");

    let initfs_slice =
        unsafe { core::slice::from_raw_parts(kbh.initfs_ptr.0 as *const u8, kbh.initfs_ptr.1) };
    let dummy_elf = Tar::new(initfs_slice)
        .iter()
        .find(|h| h.is_file("dummy"))
        .unwrap()
        .file()
        .unwrap();

    let elf = ElfOwned::new_from_slice(dummy_elf);
    let idk = unsafe { Virt2PhysMapping::inhearit_bootloader() }.unwrap();
    unsafe { idk.clone().load() };

    let process = Process::new(0, &idk);
    unsafe { process.load_tables() };
    process.add_elf(elf).unwrap();
    process
        .add_anon(
            VmRegion::from_containing(
                VirtAddr::new(0x00090000000),
                VirtAddr::new(0x00090000000 + PAGE_4K * 20),
            ),
            VmPermissions::none()
                .set_exec_flag(true)
                .set_read_flag(true)
                .set_write_flag(true)
                .set_user_flag(true),
        )
        .unwrap();
    process
        .add_anon(
            VmRegion::from_containing(
                VirtAddr::new(0x200000000000 - (10 * PAGE_4K)),
                VirtAddr::new(0x200000000000 + PAGE_4K),
            ),
            VmPermissions::none()
                .set_exec_flag(true)
                .set_read_flag(true)
                .set_write_flag(true)
                .set_user_flag(false),
        )
        .unwrap();
    process
        .add_anon(
            VmRegion::from_containing(
                VirtAddr::new(0x300000000000 - (10 * PAGE_4K)),
                VirtAddr::new(0x300000000000 + PAGE_4K),
            ),
            VmPermissions::none()
                .set_exec_flag(true)
                .set_read_flag(true)
                .set_write_flag(true)
                .set_user_flag(false),
        )
        .unwrap();
    process
        .add_anon(
            VmRegion::from_containing(
                VirtAddr::new(0x400000000000),
                VirtAddr::new(0x400000000000 + PAGE_4K),
            ),
            VmPermissions::none()
                .set_exec_flag(true)
                .set_read_flag(true)
                .set_write_flag(true)
                .set_user_flag(false),
        )
        .unwrap();
    unsafe { process.load_tables() };
    logln!("{:#?}", process);

    let mut test = ProcessContext {
        r15: 0,
        r14: 0,
        r13: 0,
        r12: 0,
        r11: 0,
        r10: 0,
        r9: 0,
        r8: 0,
        rbp: 0,
        rdi: 0,
        rsi: 0,
        rdx: 0,
        rcx: 0,
        rbx: 0,
        rax: 0,
        cs: (5 << 3) | 3,
        ss: (4 << 3) | 3,
        rflag: 0x200,
        rip: 0x00400000,
        exception_code: 0,
        rsp: (0x00090000000 + PAGE_4K * 20) as u64,
    };

    logln!("Attempting to jump to userspace! -- {:#016x?}", test);
    unsafe { context::userspace_entry(&raw mut test) };
    loop {
        logln!("FINISH");
    }
}
