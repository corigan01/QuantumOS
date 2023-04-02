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
use core::fmt::Debug;
use core::panic::PanicInfo;
use quantum_lib::simple_allocator::SimpleAllocator;

use stage_1::bios_disk::BiosDisk;
use stage_1::bios_ints::{BiosInt, TextModeColor};
use stage_1::config_parser::BootloaderConfig;
use stage_1::error::BootloaderError;
use stage_1::filesystem::FileSystem;
use stage_1::vesa::{BasicVesaController, VesaModeInfo};
use stage_1::{bios_print, bios_println};

global_asm!(include_str!("init.s"));

#[no_mangle]
extern "C" fn bit16_entry(disk_number: u16) {
    enter_rust(disk_number);
    panic!("Stage1 should not return!");
}

static mut TEMP_ALLOC: Option<SimpleAllocator> = None;

fn enter_rust(disk_id: u16) {
    bios_println!("\n --- Quantum Boot loader 16 ---\n");

    unsafe {
        TEMP_ALLOC = SimpleAllocator::new_from_ptr(0x00100000 as *mut u8, 0x03200000);
    }

    let fs =
        FileSystem::<BiosDisk>::new(BiosDisk::new(disk_id as u8))
            .toggle_logging()
            .quarry_disk()
            .expect("Could not read any supported filesystems!")
            .mount_root_if_contains("/bootloader/bootloader.cfg")
            .expect("Could detect bootloader partition, please add \'/bootloader/bootloader.cfg\' to the bootloader filesystem for a proper boot!");

    let bootloader_config_file =
        unsafe { TEMP_ALLOC.as_mut().unwrap().allocate_region(256).unwrap() };
    let bootloader_filename = "/bootloader/bootloader.cfg";

    fs.load_file_into_slice(bootloader_config_file, bootloader_filename)
        .expect("Unable to load bootloader config file!");

    let bootloader_config =
        BootloaderConfig::from_str(core::str::from_utf8(bootloader_config_file).unwrap())
            .expect("Unable to parse bootloader config!");

    bios_println!("{:#?}", bootloader_config);

    let next_stage = unsafe {
        TEMP_ALLOC
            .as_mut()
            .unwrap()
            .allocate_region(
                fs.get_filesize_bytes(bootloader_config.get_stage2_file_path())
                    .expect("Could not get stage2 filesize")
                    + 0x10,
            )
            .unwrap()
    };

    fs.load_file_into_slice(next_stage, bootloader_config.get_stage2_file_path())
        .expect("Could not load next stage!");

    let kernel = unsafe {
        TEMP_ALLOC
            .as_mut()
            .unwrap()
            .allocate_region(
                fs.get_filesize_bytes(bootloader_config.get_kernel_file_path())
                    .expect("Could not get kernel filesize")
                    + 0x10,
            )
            .unwrap()
    };

    fs.load_file_into_slice(kernel, bootloader_config.get_kernel_file_path())
        .expect("Could not load kernel!");

    let bootloader_video_mode = bootloader_config.get_video_info();

    if let Ok(vga_info) = BasicVesaController::new() {
        let mode = vga_info.run_on_every_supported_mode_and_return_on_true(|mode, number| {
            let x = mode.width as usize;
            let y = mode.height as usize;

            x == bootloader_video_mode.0 && y == bootloader_video_mode.1
        });

        bios_println!("\nPicked mode {:?}", &mode);

        vga_info.set_video_mode(mode.unwrap()).unwrap();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    bios_println!("{}", info);

    loop {}
}
