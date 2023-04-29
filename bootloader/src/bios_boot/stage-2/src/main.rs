/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
  Part of the Quantum OS Project

Copyright 2023 Gavin Kellam

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
#![no_main] // disable all Rust-level entry points
#![no_std] // don't link the Rust standard library
#![allow(dead_code)]

use core::arch::asm;
use core::panic::PanicInfo;

use quantum_lib::bytes::Bytes;
use quantum_lib::debug::add_connection_to_global_stream;
use quantum_lib::debug::stream_connection::StreamConnectionBuilder;
use quantum_lib::debug_println;
use quantum_lib::x86_64::{PrivlLevel};
use quantum_lib::x86_64::registers::{CpuStack, Segment, SegmentRegs};

use stage_2::debug::{display_string, setup_framebuffer};
use stage_2::gdt::LONG_MODE_GDT;
use stage_2::paging::enable_paging;

use bootloader::boot_info::{BootInfo, VideoInformation};

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: u32) -> ! {
    let boot_info_ref = unsafe { &*(boot_info as *const BootInfo) };

    let video_info: &VideoInformation = boot_info_ref.vid.as_ref().unwrap();

    let framebuffer = video_info.framebuffer;
    let x_res = video_info.x;
    let y_res = video_info.y;
    let bbp = video_info.depth;

    setup_framebuffer(
        framebuffer,
        x_res as usize,
        y_res as usize,
        bbp as usize,
        true,
    );

    let stream_connection = StreamConnectionBuilder::new()
        .console_connection()
        .add_outlet(display_string)
        .add_connection_name("VGA DEBUG")
        .does_support_scrolling(true)
        .build();
    add_connection_to_global_stream(stream_connection).unwrap();

    debug_println!("Quantum Bootloader! (Stage2) [32 bit]");

    main(boot_info_ref);
    panic!("Stage2 should not finish!");
}

fn main(boot_info: &BootInfo) {
    let mut total_memory = 0;
    for entry in boot_info.memory_map.unwrap() {
        if entry.len == 0 && entry.address == 0 {
            break;
        }

        if entry.entry_type == 1 {
            total_memory += entry.len;
        }
    }

    debug_println!(
        "Memory Avl: {:?} {}",
        boot_info.memory_map.unwrap().as_ptr(),
        Bytes::from(total_memory)
    );

    debug_println!("Vga info: {:#?}", boot_info.vid);

    unsafe { enable_paging() };
    LONG_MODE_GDT.load();

    let ptr = boot_info.ram_fs.unwrap().stage3.ptr;
    let data_ref = unsafe { &*(ptr as *const [u8; 10]) };

    debug_println!("Entering Stage3! 0x{:x} {:x?}", ptr, data_ref);

    unsafe { enter_stage3(boot_info); }
}

#[no_mangle]
pub unsafe fn enter_stage3(boot_info: &BootInfo) {
    SegmentRegs::reload_all_to(Segment::new(2, PrivlLevel::Ring0));

    CpuStack::push(0);
    CpuStack::push(boot_info as *const BootInfo as u32 - 8);
    CpuStack::push(0);
    CpuStack::push(boot_info.ram_fs.unwrap().stage3.ptr as u32);

    asm!("ljmp $0x8, $2f", "2:", options(att_syntax));
    asm!(
        ".code64",
        // jump to 3rd stage
        "pop rax",
        "pop rdi",

        "call rax",

        "2:",
        "jmp 2b",
        in("rax") 0,
        in("rdi") 0
    );

}



#[panic_handler]
#[cold]
#[allow(dead_code)]
fn panic(info: &PanicInfo) -> ! {
    debug_println!("\nBootloader PANIC\n{}", info);
    loop {}
}
