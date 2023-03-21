/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(quantum_os::test_handler::test_runner)]

//mod vga;
use owo_colors::OwoColorize;
use bootloader::boot_info::{BootInfo, MemoryRegionKind};

#[cfg(not(test))]
use bootloader::entry_point;
use lazy_static::lazy_static;

use quantum_os::arch_x86_64::idt::{interrupt_tester, set_quiet_interrupt, InterruptFrame};
use quantum_os::arch_x86_64::isr::general_isr;
use quantum_os::arch_x86_64::{INTERRUPT_DT};
use quantum_os::debug_output;
use quantum_os::debug_output::StreamInfo;
use quantum_os::serial::SERIAL1;
use quantum_os::vga::low_level::FBuffer;
use quantum_os::{attach_interrupt};
use quantum_os::{debug_print, debug_println};
use quantum_os::memory::physical_memory::{PhyRegionKind, PhyRegion, PhyRegionMap};
use quantum_os::memory::pmm::PhyMemoryManager;
use quantum_os::memory::init_alloc::INIT_ALLOC;
use quantum_os::vga::framebuffer::RawColor;
use quantum_os::clock::rtc;
use quantum_os::clock::rtc::{set_time_zone, update_and_get_time};

fn debug_output_char(char: u8) {
    if let Some(serial_info) = SERIAL1.lock().as_ref() {
        serial_info.write_byte(char);
    }
}

#[cfg(not(test))]
entry_point!(main);

//#[cfg(not(test))]
fn main(boot_info: &'static mut BootInfo) -> ! {

    // safely get the baud rate
    let baud_rate = if let Some(serial) = SERIAL1.lock().as_ref() {
        serial.get_baud()
    } else {
        0
    };

    // set the debug stream to the serial output
    debug_output::set_debug_stream(StreamInfo {
        output_stream: Some(debug_output_char),
        name: Some("Serial"),
        speed: Some(baud_rate as u64),
        color: true,
        message_header: true,
    });

    debug_println!("\nQuantum Hello!");

    set_time_zone(-6);

    debug_println!("Time: {}", update_and_get_time());

    debug_println!("\n{:#x?}\n", boot_info);
    debug_print!("{}", "Checking the framebuffer ... ".white());

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        for byte in framebuffer.buffer_mut() {
            *byte = 0x0F;
        }
        for byte in framebuffer.buffer_mut() {
            *byte = 0x00;
        }
        for byte in framebuffer.buffer_mut() {
            *byte = 0x0F;
        }

        debug_println!("{}", "OK".bright_green().bold());
    }
    else {
        debug_println!("{}", "FAIL".bright_red().bold());
    }


    // init the cpu
    {
        // init the cpu, we just need to wake up the lazy_statics for them to init
        let mut idt = INTERRUPT_DT.lock();

        attach_interrupt!(idt, general_isr, 0..32);

        set_quiet_interrupt(1, true);

        idt.submit_entries().expect("Failed to load IDT!").load();

        debug_print!("{}", "Testing Interrupts ... ".white());

        interrupt_tester();

        debug_println!("{}", "OK".bright_green().bold());
    }

    let mut pmm = PhyMemoryManager::new();

    // init memory regions
    {
        debug_println!("\n\nBoot Memory Info:");

        let memory_info = &boot_info.memory_regions;
        let mut region_map = PhyRegionMap::new();

        let kernel_ptr = INIT_ALLOC.lock().alloc(1).unwrap() as *const u8 as u64;

        debug_println!("|      START      |       END       |         KIND       |");
        debug_println!("|-----------------|-----------------|--------------------|");

        for i in 0..memory_info.len() {
            let memory_region = memory_info[i];
            let memory_entry = PhyRegion::new()
                .set_type(
                    if memory_region.kind == MemoryRegionKind::Usable
                    { PhyRegionKind::Usable} else { PhyRegionKind::NotUsable})
                .set_address_range(memory_region.start..memory_region.end);

            match memory_region.kind {
                MemoryRegionKind::Usable => {
                    debug_print!("| {:#15X} | {:#15X} | \t{:?}  \t |",
                        memory_region.start, memory_region.end, memory_region.kind.bold().green());
                }
                MemoryRegionKind::Bootloader => {
                    debug_print!("| {:#15X} | {:#15X} | \t{:?}\t |",
                        memory_region.start, memory_region.end, memory_region.kind.yellow());
                }
                _ => {
                    debug_print!("| {:#15X} | {:#15X} | \t{:?}\t |",
                        memory_region.start, memory_region.end, memory_region.kind.red());
                }
            }

            if memory_region.start <= kernel_ptr && memory_region.end >= kernel_ptr {
                debug_println!("  <-- Kernel");

                pmm.set_kernel_region(memory_entry);
            } else {
                debug_print!("\n");
            }

            region_map.add_entry(memory_entry);
        }

        debug_println!("|-----------------|-----------------|--------------------|");

        debug_println!("Does the memory overlap       : {}", region_map.do_regions_overlap());

        let free_bytes = region_map.get_total_bytes(PhyRegionKind::Usable);
        let free_pages = region_map.get_usable_pages();

        let total_size: u64 = if let Some(regions) = region_map.get_regions(PhyRegionKind::Usable) {
            let mut end_of_map: u64 = 0;
            for i in regions {
                if i.end.as_u64() > end_of_map {
                    end_of_map = i.end.as_u64();
                }
            }

            end_of_map
        } else { 0 };

        debug_println!("Total Free Physical Memory    : {} {} ({} MB)",
            free_pages.green().bold(),
            "Pages".green().bold(),
            free_bytes / (1024 * 1024));

        debug_println!("Recovered Memory Information  : {} MB Usable / {} MB Total -- {}% Usable",
            (free_bytes / (1024 * 1024)).white().bold(),
            (total_size / (1024 * 1024)).white().bold(),
            (((free_bytes as f64 / total_size as f64) * 100 as f64) as u64));

        let free_regions = region_map.get_regions(PhyRegionKind::Usable)
            .expect("Unable to find any free memory regions! Unable to boot!");

        debug_println!("Amount of free elements       : {}/{} ", free_regions.len(), free_regions.capacity());

        for i in free_regions {
            let recommended_size =
                PhyMemoryManager::recommended_bytes_to_store_allocation(i);

            let alloc = INIT_ALLOC.lock().alloc(recommended_size)
                .expect("Unable to allocate region for Allocation");

            pmm.insert_new_region(i, unsafe { &mut *alloc })
                .expect("Unable to add region to PMM");
        }
    }

    debug_println!("Remaining InitAlloc           : {} Bytes",
        INIT_ALLOC.lock().remaining_capacity().white().bold());

    let kernel_buffer = FBuffer::new(&boot_info.framebuffer);

    kernel_buffer.draw_rec((000, 000), (100, 100), 0xFF0000);
    kernel_buffer.draw_rec((100, 100), (100, 100), 0x00FF00);
    kernel_buffer.draw_rec((200, 200), (100, 100), 0x0000FF);

    debug_println!("\n\n\n==== KERNEL MAIN FINISHED ==== ");
    debug_println!("In later versions of this kernel, the kernel should not finish!");

    // Make a little color changing box on screen to let the user know
    // that the kernel is still alive and running.
    let mut x: i32 = 0;
    let mut sign: i32 = 1;
    loop {
        if x == 255 {
            sign = -1;
        }
        if x == 0 {
            sign = 1;
        }
        x += sign;

        let color = (x as u32) << 16 | (x as u32) << 8 | (x as u32);
        kernel_buffer.draw_rec((300, 300), (100, 100), color);
    }
}