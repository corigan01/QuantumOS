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
use bootloader::boot_info::BootInfo;
use quantum_lib::com::serial::{SerialBaud, SerialDevice, SerialPort};
use quantum_lib::debug::{add_connection_to_global_stream};
use quantum_lib::debug::stream_connection::StreamConnectionBuilder;
use quantum_lib::debug_println;
use quantum_os::clock::rtc::{set_time_zone, update_and_get_time};

static mut SERIAL_CONNECTION: Option<SerialDevice> = None;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info_ptr: u64) {
    let connection = unsafe { &mut SERIAL_CONNECTION };

    *connection = Some(SerialDevice::new(SerialPort::Com1, SerialBaud::Baud115200).unwrap());
    let connection = StreamConnectionBuilder::new()
        .console_connection()
        .add_connection_name("SERIAL")
        .does_support_scrolling(true)
        .add_outlet(unsafe { SERIAL_CONNECTION.as_ref().unwrap() })
        .build();

    add_connection_to_global_stream(connection).unwrap();

    set_time_zone(7);

    debug_println!("Welcome to Quantum OS! {}\n", update_and_get_time());
    debug_println!("Bootloader info ptr 0x{:x}", boot_info_ptr);

    let bootloader_info = BootInfo::from_ptr(boot_info_ptr as usize);

    main(bootloader_info);

    panic!("Kernel Should not exit!!!");
}

fn main(boot_info: &BootInfo) {
    debug_println!("{:#?}", boot_info.get_video_information());

}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug_println!("{}", info);
    loop {}
}