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

use core::panic::PanicInfo;

use quantum_lib::{debug_println, kernel_entry};
use quantum_lib::address_utils::region::MemoryRegionType;
use quantum_lib::boot::boot_info::KernelBootInformation;
use quantum_lib::bytes::Bytes;
use quantum_lib::com::serial::{SerialBaud, SerialDevice, SerialPort};
use quantum_lib::debug::add_connection_to_global_stream;
use quantum_lib::debug::stream_connection::StreamConnectionBuilder;
use quantum_lib::gfx::{Pixel};

use quantum_os::clock::rtc::update_and_get_time;

static mut SERIAL_CONNECTION: Option<SerialDevice> = None;

kernel_entry!(main);

fn main(boot_info: &KernelBootInformation) {
    let connection = unsafe { &mut SERIAL_CONNECTION };
    *connection = Some(SerialDevice::new(SerialPort::Com1, SerialBaud::Baud115200).unwrap());

    let connection = StreamConnectionBuilder::new()
        .console_connection()
        .add_connection_name("SERIAL")
        .does_support_scrolling(true)
        .add_outlet(unsafe { SERIAL_CONNECTION.as_ref().unwrap() })
        .build();

    add_connection_to_global_stream(connection).unwrap();

    debug_println!("Welcome to Quantum OS! {}\n", update_and_get_time());

    let mut physical_memory_map = boot_info.get_physical_memory().clone();
    let mut virtual_memory_map = boot_info.get_virtual_memory().clone();

    physical_memory_map.consolidate().unwrap();
    virtual_memory_map.consolidate().unwrap();

    debug_println!("Virtual Memory Map:\n{virtual_memory_map:?}");
    debug_println!("Physical Memory Map:\n{physical_memory_map:?}");

    let total_phy: u64 = boot_info.physical_regions.total_mem_for_type(MemoryRegionType::Usable).into();
    let total_pages: u64 = total_phy / (4 * Bytes::KIB);

    debug_println!("Total Usable Physical Memory {} ({} -- 4k Pages)",
        Bytes::from(total_phy),
        total_pages
    );

    let mut framebuffer = boot_info.framebuffer;

    framebuffer.fill_entire(Pixel::from_hex(0x111111));
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug_println!("{}", info);
    loop {}
}