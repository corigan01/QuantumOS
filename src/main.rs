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

use core::arch::asm;
use core::borrow::Borrow;
use core::fmt::Write;
use core::panic::PanicInfo;

//mod vga;
use bootloader::boot_info::{BootInfo, FrameBuffer, MemoryRegion};
use bootloader::entry_point;
use owo_colors::OwoColorize;

use quantum_os::arch_x86_64::idt::{interrupt_tester, set_quiet_interrupt, InterruptFrame};
use quantum_os::arch_x86_64::isr::general_isr;
use quantum_os::arch_x86_64::{GLOBAL_DT, INTERRUPT_DT};
use quantum_os::debug_output;
use quantum_os::debug_output::StreamInfo;
use quantum_os::serial::SERIAL1;
use quantum_os::vga::low_level::FBuffer;
use quantum_os::{attach_interrupt, remove_interrupt, serial_print, serial_println};
use quantum_os::{bitset, debug_print, debug_println};

fn debug_output_char(char: u8) {
    if let Some(serial_info) = SERIAL1.lock().as_ref() {
        serial_info.write_byte(char);
    }
}

#[cfg(not(test))]
entry_point!(main);

#[cfg(not(test))]
fn main(boot_info: &'static mut BootInfo) -> ! {
    // safely get the baud rate
    let baud_rate = if let Some(serial) = SERIAL1.lock().as_ref() {
        serial.get_baud()
    } else {
        0
    };

    // set the debug stream to the serial output
    debug_output::set_stream(StreamInfo {
        output_stream: Some(debug_output_char),
        name: Some("Serial"),
        speed: Some(baud_rate as u64),
        color: true,
        message_header: true,
    });

    debug_println!("\n{:#?}\n", boot_info);
    debug_print!("Checking the framebuffer ... ");

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        for byte in framebuffer.buffer_mut() {
            *byte = 0x0F;
        }

        debug_println!("{}", "OK".bright_green().bold());
    } else {
        debug_println!("FAIL");
    }

    {
        // init the cpu, we just need to wake up the lazy_statics for them to init
        let mut idt = INTERRUPT_DT.lock();

        attach_interrupt!(idt, general_isr, 0..32);

        set_quiet_interrupt(1, true);

        idt.submit_entries().expect("Failed to load IDT!").load();

        debug_print!("Testing Interrupts ... ");

        interrupt_tester();

        debug_println!("OK");
    }

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
