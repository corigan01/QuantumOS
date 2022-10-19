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

Quantum OS Lib file, documentation coming soon!

*/

#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]

#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_handler::test_runner)]
#![feature(abi_x86_interrupt)]

#![reexport_test_harness_main = "run_test"]

#[cfg(test)]
use bootloader::{BootInfo, entry_point};
use owo_colors::OwoColorize;
use crate::debug_output::{set_debug_stream, StreamInfo};
use crate::serial::SERIAL1;

#[cfg(test)]
entry_point!(test_main);

#[cfg(test)]
fn debug_output_char(char: u8) {
    if let Some(serial_info) = SERIAL1.lock().as_ref() {
        serial_info.write_byte(char);
    }
}

#[cfg(test)]
/// Entry point for `cargo test`
fn test_main(boot_info: &'static mut BootInfo) -> ! {
    let baud_rate = if let Some(serial) = SERIAL1.lock().as_ref() {
        serial.get_baud()
    } else {
        0
    };

    set_debug_stream(StreamInfo {
        output_stream: Some(debug_output_char),
        name: Some("Serial"),
        speed: Some(baud_rate as u64),
        color: true,
        message_header: false,
    });

    debug_println!("\n\n{} {}\n",
        "                                                     ",
        "QuantumOS in Test Mode".bright_green().bold());

    run_test();

    loop {};
}

pub mod test_handler;

pub mod memory_utils;
pub mod serial;
pub mod vga;
pub mod port;
pub mod arch_x86_64;
pub mod memory;
pub mod bitset;
pub mod panic;
pub mod qemu;
pub mod post_hal;
pub mod debug_output;
pub mod error_utils;