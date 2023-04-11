#![feature(panic_info_message)]
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

#[cfg(debug)]
use stage_1::bios_println;

use bootloader::bios_call::BiosCall;
use bootloader::boot_info::{BootInfo, SimpleRamFs};
use bootloader::BootMemoryDescriptor;
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use quantum_lib::simple_allocator::SimpleBumpAllocator;
use stage_1::bios_disk::BiosDisk;
use stage_1::bios_video::{BiosTextMode, _print};
use stage_1::config_parser::BootloaderConfig;
use stage_1::filesystem::FileSystem;
use stage_1::unreal::{enter_stage2, enter_unreal_mode};
use stage_1::vesa::{BiosVesa, Res};

global_asm!(include_str!("init.s"));

#[no_mangle]
extern "C" fn bit16_entry(disk_number: u16) {
    enter_rust(disk_number);
}

#[link_section = ".GDT"]
static mut TEMP_ALLOC: Option<SimpleBumpAllocator> = None;

fn enter_rust(disk_id: u16) {
    BiosTextMode::print_int_bytes(b"Quantum Bootloader (Stage1)\n");

    BiosTextMode::print_int_bytes(b"Unreal ...");
    unsafe {
        enter_unreal_mode();
    };
    BiosTextMode::print_int_bytes(b" OK\n");

    let mut boot_info = BootInfo::default();

    unsafe {
        TEMP_ALLOC = SimpleBumpAllocator::new_from_ptr((0x00100000 - (512)) as *mut u8, 0x03200000);
    }

    if unsafe { &TEMP_ALLOC }.is_none() {
        BiosTextMode::print_int_bytes(b"Allocator is broken :(");
    }

    BiosTextMode::print_int_bytes(b"Loading files ");
    let fs =
        FileSystem::new(BiosDisk::new(disk_id as u8))
            .quarry_disk()
            .expect("Could not read any supported filesystems!")
            .mount_root_if_contains("/bootloader/bootloader.cfg")
            .expect("Could detect bootloader partition, please add \'/bootloader/bootloader.cfg\' to the bootloader filesystem for a proper boot!");

    let bootloader_config_file_ptr =
        unsafe { TEMP_ALLOC.as_mut().unwrap().allocate_region(511).unwrap() };
    let bootloader_filename = "/bootloader/bootloader.cfg";

    fs.load_file_into_slice(bootloader_config_file_ptr, bootloader_filename)
        .expect("Unable to load bootloader config file!");

    BiosTextMode::print_int_bytes(b"...config...");
    let bootloader_config_string =
        unsafe { core::str::from_utf8_unchecked(bootloader_config_file_ptr) };

    let bootloader_config = BootloaderConfig::from_str(bootloader_config_string)
        .expect("Unable to parse bootloader config!");

    let next_stage_filesize_bytes = fs
        .get_filesize_bytes(bootloader_config.get_stage2_file_path())
        .expect("Could not get stage2 filesize");

    let next_stage_ptr = unsafe {
        TEMP_ALLOC
            .as_mut()
            .unwrap()
            .allocate_region(next_stage_filesize_bytes + 0x10)
            .unwrap()
    };

    let kernel_filesize_bytes = fs
        .get_filesize_bytes(bootloader_config.get_kernel_file_path())
        .expect("Could not get kernel filesize");

    let kernel_ptr = unsafe {
        TEMP_ALLOC
            .as_mut()
            .unwrap()
            .allocate_region(kernel_filesize_bytes + 0x10)
            .unwrap()
    };

    BiosTextMode::print_int_bytes(b"...stage2...");
    fs.load_file_into_slice(next_stage_ptr, bootloader_config.get_stage2_file_path())
        .expect("Could not load next stage!");

    BiosTextMode::print_int_bytes(b"...kernel...");
    fs.load_file_into_slice(kernel_ptr, bootloader_config.get_kernel_file_path())
        .expect("Could not load kernel!");

    BiosTextMode::print_int_bytes(b" OK\n");

    boot_info.ram_fs = Some(SimpleRamFs::new(
        BootMemoryDescriptor {
            ptr: kernel_ptr.as_ptr() as u64,
            size: kernel_filesize_bytes as u64,
        },
        BootMemoryDescriptor {
            ptr: next_stage_ptr.as_ptr() as u64,
            size: next_stage_filesize_bytes as u64,
        },
    ));

    //BiosTextMode::print_int_bytes(b"\n");
    //BiosTextMode::print_int_bytes(bootloader_config_string.as_bytes());
    //BiosTextMode::print_int_bytes(b"\n");

    BiosTextMode::print_int_bytes(b"Getting Vesa Info ... ");
    let raw_video_info = bootloader_config.get_recommended_video_info();
    let expected_res = Res {
        x: raw_video_info.0,
        y: raw_video_info.1,
        depth: 24,
    };

    let mut vesa = BiosVesa::new().quarry().unwrap();
    let closest_mode = vesa.find_closest_mode(expected_res).unwrap();
    BiosTextMode::print_int_bytes(b"OK\n");

    BiosTextMode::print_int_bytes(b"Setting Mode ... ");
    //vesa.set_mode(closest_mode).unwrap();
    BiosTextMode::print_int_bytes(b"OK\n");

    BiosTextMode::print_int_bytes(b"Loading Stage2\n");
    unsafe {
        enter_stage2(
            next_stage_ptr.as_ptr(),
            &boot_info as *const BootInfo as *const u8,
        );
    }
}

#[panic_handler]
#[cold]
#[allow(dead_code)]
fn panic(info: &PanicInfo) -> ! {
    _print(format_args!("{}", info));

    loop {}
}
