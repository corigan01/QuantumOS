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
#![test_runner(quantum_os::test_handler::test_runner)]

//mod vga;
use core::arch::asm;
use core::panic::PanicInfo;
use bootloader::boot_info::{BootInfo, FrameBuffer, MemoryRegion};
use bootloader::entry_point;

use quantum_os::{serial_println, serial_print};
use quantum_os::arch_x86_64::{set_up_gdt, set_up_idt};
use quantum_os::serial::SERIAL1;
use quantum_os::vga::low_level::FBuffer;
use quantum_os::bitset;

#[cfg(not(test))]
entry_point!(main);

#[cfg(not(test))]
fn main(boot_info: &'static mut BootInfo) -> ! {
    serial_println!("\n\n");
    serial_println!("--- Quantum is using this serial port for debug information ---");
    serial_println!("---       Baud rate is set at 'None' bits per second        ---");

    serial_println!("\n{:#?}\n", boot_info);

    serial_print!("Checking the framebuffer ... ");

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        for byte in framebuffer.buffer_mut() {
            *byte = 0x0F;
        }

        serial_println!("OK");
    }
    else { serial_println!("FAIL"); }

    // setup cpu
    serial_print!("Setting up GDT ... "); set_up_gdt(); serial_println!("OK");
    serial_print!("Setting up IDT ... "); set_up_idt(); serial_println!("OK");


    let kernel_buffer = FBuffer::new(&boot_info.framebuffer);

    kernel_buffer.draw_rec((000, 000), (100, 100), 0xFF0000);
    kernel_buffer.draw_rec((100, 100), (100, 100), 0x00FF00);
    kernel_buffer.draw_rec((200, 200), (100, 100), 0x0000FF);


    serial_println!("\n\n\n==== KERNEL MAIN FINISHED ==== ");
    serial_println!("In later versions of this kernel, the kernel should not finish!");
    loop {}
}
