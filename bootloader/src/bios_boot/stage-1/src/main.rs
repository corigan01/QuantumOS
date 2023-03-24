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

#![no_main]
#![no_std]

use core::arch::global_asm;
use core::panic::PanicInfo;

use stage_1::bios_disk::BiosDisk;
use stage_1::bios_ints::{BiosInt, TextModeColor};
use stage_1::cstring::{CStringOwned, CStringRef};
use stage_1::vesa::BasicVesaInfo;
use stage_1::{bios_print, bios_println};
use stage_1::config_parser::BootloaderConfig;
use stage_1::filesystem::FileSystem;

global_asm!(include_str!("init.s"));

#[no_mangle]
extern "C" fn bit16_entry(disk_number: u16) {
    enter_rust(disk_number);
    panic!("Stage should not return!");
}

fn enter_rust(disk: u16) {
    bios_println!("\n --- Quantum Boot loader 16 ---\n");

    bios_println!("Attempting to find valid filesystems!");
    let fs =
        FileSystem::<BiosDisk>::new(BiosDisk::new(disk as u8))
            .quarry_disk()
            .expect("Could not read any supported filesystems!")
            .mount_root_if_contains("/bootloader/bootloader.cfg")
            .expect("Could detect bootloader partition, please add \'/bootloader/bootloader.cfg\' to the bootloader filesystem for a proper boot!");

    bios_println!("Mounted bootloader partition");
    bios_println!("Reading file");

    // FIXME: maybe should be an allocated buffer in the future
    let mut bootloader_config_buffer = [0_u8; 1024];

    unsafe {
        fs.read_file_into_buffer(bootloader_config_buffer.as_mut_ptr(), "/bootloader/bootloader.cfg")
    }.expect("Unable to load config file!");

    bios_println!("CONFIG:\n{}", CStringRef::from_bytes(&bootloader_config_buffer) );



    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    bios_println!("{}", info);

    loop {}
}
