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

use core::arch::asm;
use core::panic::PanicInfo;
use bootloader::boot_info::BootInfo;

use quantum_lib::debug::{add_connection_to_global_stream, StreamableConnection};
use quantum_lib::debug::stream_connection::StreamConnectionBuilder;
use quantum_lib::{debug_print, debug_println};
use quantum_lib::com::serial::{SerialBaud, SerialDevice, SerialPort};
use quantum_lib::elf::{ElfHeader, ElfArch, ElfBits, ElfSegmentType};
use quantum_lib::x86_64::PrivlLevel;
use quantum_lib::x86_64::registers::{Segment, SegmentRegs};

use stage_3::debug::{clear_framebuffer, display_string, setup_framebuffer};

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: u64) -> ! {
    let boot_info_ref = BootInfo::from_ptr(boot_info as usize);

    unsafe { SegmentRegs::set_data_segments(Segment::new(2, PrivlLevel::Ring0)); }

    let video_info = boot_info_ref.get_video_information();

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

    clear_framebuffer();

    let stream_connection = StreamConnectionBuilder::new()
        .console_connection()
        .add_simple_outlet(display_string)
        .add_connection_name("VGA DEBUG")
        .does_support_scrolling(true)
        .build();

    add_connection_to_global_stream(stream_connection).unwrap();

    debug_println!("Quantum Bootloader! (Stage3) [64 bit]");

    debug_println!("{:#?}", boot_info_ref.get_video_information());

    main(boot_info_ref);
    panic!("Stage3 should not return!");
}

fn parse_kernel_elf(kernel_elf: ElfHeader) -> Option<u32> {
    let kernel_arch = kernel_elf.elf_arch()?;
    let kernel_bits = kernel_elf.elf_bits()?;

    debug_print!("Kernel arch={:?}, bits={:?} ...", kernel_arch, kernel_bits);

    if !matches!(kernel_arch, ElfArch::X86_64) || !matches!(kernel_bits, ElfBits::Bit64) {
        debug_println!("Err");
        return None;
    }

    debug_println!("OK");

    let header_amount = kernel_elf.elf_number_of_entries_in_program_table()?;
    let entry_point = kernel_elf.elf_entry_point()? as u32;

    debug_println!(
        "Kernel Info: p-header=(O: {} S: {} B: {}) s-header=(O: {}, S: {}, B: {}) e-point={:x}",
        kernel_elf.elf_program_header_table_position()?,
        header_amount,
        kernel_elf.elf_size_of_entry_in_program_table()?,
        kernel_elf.elf_section_header_table_position()?,
        kernel_elf.elf_number_of_entries_in_section_table()?,
        kernel_elf.elf_size_of_entry_in_section_table()?,
        entry_point
    );

    for i in 0..header_amount {
        let header_idx = i as usize;
        let header = kernel_elf.get_program_header(header_idx)?;

        debug_print!(
            "Header {} = '{:x?}' -- {:?} => F: {} M: {} O: {} Vaddr: {:x}",
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

            debug_println!("      LOADED");

        } else {
            debug_println!("       SKIP");
        }
    }

    Some(entry_point)
}

fn main(boot_info: &BootInfo) {
    let boot_info_ptr = boot_info as *const BootInfo as u64;

    debug_println!("Starting to parse kernel ELF...");

    let kernel_info = boot_info.get_kernel_entry();
    let kernel_slice = unsafe { core::slice::from_raw_parts_mut(kernel_info.ptr as *mut u8, kernel_info.size as usize) };

    let kernel_elf = ElfHeader::from_bytes(kernel_slice).unwrap();

    let entry_point = parse_kernel_elf(kernel_elf).unwrap();

    debug_println!("Kernel Entry Point 0x{:x}", entry_point);

    debug_println!("Calling Kernel!");
    clear_framebuffer();
    unsafe {
        asm!(
            "jmp {kern:r}",
            in("rdi") boot_info_ptr,
            kern = in(reg) entry_point,
        );
    }
}

#[panic_handler]
#[cold]
#[allow(dead_code)]
fn panic(info: &PanicInfo) -> ! {
    fn outlet(msg: &str) {
        SerialDevice::new(SerialPort::Com1, SerialBaud::Baud115200).unwrap().display_string(msg);
    }

    let stream_connection = StreamConnectionBuilder::new()
        .console_connection()
        .add_simple_outlet(outlet)
        .add_connection_name("SERIAL ERROR")
        .does_support_scrolling(true)
        .build();

    add_connection_to_global_stream(stream_connection).unwrap();

    debug_println!("\nStage-3 PANIC ============\n{}\n\n", info);
    loop {}
}