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

#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]

use core::arch::asm;
use core::panic::PanicInfo;
use bootloader::boot_info::{BootInfo, VideoInformation};
use quantum_lib::debug::add_connection_to_global_stream;
use quantum_lib::debug::stream_connection::StreamConnectionBuilder;
use quantum_lib::{debug_print, debug_println};
use quantum_lib::bytes::Bytes;
use quantum_lib::elf::{ElfArch, ElfBits, ElfHeader, ElfSegmentType};
use quantum_lib::ptr::entry_point::EntryPoint64;
use quantum_lib::x86_64::registers::SegmentRegs;

use stage_2::debug::{display_string, setup_framebuffer};
use stage_2::gdt::LONG_MODE_GDT;
use stage_2::paging::enable_paging;

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

    debug_println!("Quantum Bootloader! (Stage2)");

    main(boot_info_ref);
    panic!("Stage2 should not finish!");
}

fn parse_kernel_elf(kernel_elf: ElfHeader) -> Option<()> {
    let kernel_arch = kernel_elf.elf_arch()?;
    let kernel_bits = kernel_elf.elf_bits()?;

    debug_print!("Kernel arch={:?}, bits={:?} ...", kernel_arch, kernel_bits);

    if !matches!(kernel_arch, ElfArch::X86_64) || !matches!(kernel_bits, ElfBits::Bit64) {
        debug_println!("Err");
        return None;
    }

    debug_println!("OK");

    let header_amount = kernel_elf.elf_number_of_entries_in_program_table()?;

    debug_println!("Kernel Info: p-header=(O: {} S: {} B: {}) s-header=(O: {}, S: {}, B: {}) e-point={}",
        kernel_elf.elf_program_header_table_position()?,
        header_amount,
        kernel_elf.elf_size_of_entry_in_program_table()?,
        kernel_elf.elf_section_header_table_position()?,
        kernel_elf.elf_number_of_entries_in_section_table()?,
        kernel_elf.elf_size_of_entry_in_section_table()?,
        kernel_elf.elf_entry_point()?
    );

    for i in 0..header_amount {
        let header_idx = i as usize;
        let header = kernel_elf.get_program_header(header_idx)?;

        debug_println!("Header {} = '{:x?}' -- {:?} => F: {} M: {} O: {} Vaddr: {}",
            header_idx,
            header.type_of_segment(),
            header.flags(),
            header.size_in_elf(),
            header.size_in_mem(),
            header.data_offset(),
            header.data_expected_address()
        );

        // Test code
        let header_type = header.type_of_segment();
        let kernel_raw_data = kernel_elf.get_raw_data_slice();

        if matches!(header_type, ElfSegmentType::Load) {
            let loader_ptr = header.data_expected_address() as *mut u8;

            let data_size = header.size_in_elf() as usize;
            let ac_data_offset = header.data_offset() as usize;

            let data_slice = &kernel_raw_data[ac_data_offset..(data_size + ac_data_offset)];

            for (i, byte) in data_slice.iter().enumerate() {
                unsafe {
                    *loader_ptr.add(i) = *byte;
                }
            }

        }

    }

    Some(())
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

    let kern_disc = &boot_info.ram_fs.unwrap().kernel;
    let kernel_elf_raw_data = unsafe {
        core::slice::from_raw_parts_mut(kern_disc.ptr as *mut u8, kern_disc.size as usize)
    };

    let kernel_elf = ElfHeader::from_bytes(kernel_elf_raw_data).unwrap();

    parse_kernel_elf(kernel_elf).unwrap();

    let test_bytes = unsafe {
        core::slice::from_raw_parts_mut(16777216 as *mut u8, 40)
    };

    debug_println!("Jumping to Kernel!! {:x?}", test_bytes);

    //loop {}

    unsafe {
        test(16777216)
    }

}

pub unsafe fn test(entry_point: u64) {
    debug_println!("0");
    asm!(


    // push entry point address (extended to 64 bit)
    "push 0",
    "push {entry_point:e}",

    entry_point = in(reg) entry_point as u32,
    );
    asm!("ljmp $0x8, $2f", "2:", options(att_syntax));

    SegmentRegs::reload_all_to(0x10);

    asm!(
    ".code64",

    "mov rsp, 0x0100000",

    // jump to 4th stage

    "jmp rax",

    // enter endless loop in case 4th stage returns
    "2:",
    "jmp 2b",
    out("rax") _,
    out("rdi") _,
    d = in(reg) 0x1000000
    );

}

#[panic_handler]
#[cold]
#[allow(dead_code)]
fn panic(info: &PanicInfo) -> ! {
    debug_println!("\nBootloader PANIC\n{}", info);
    loop {}
}
