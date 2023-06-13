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

use bootloader::boot_info::{BootInfo, VideoInformation};
use bootloader::e820_memory::E820Entry;
use bootloader::BootMemoryDescriptor;
use core::arch::global_asm;
use core::mem::{MaybeUninit, size_of};
use core::panic::PanicInfo;
use quantum_lib::alloc::simple_allocator::SimpleBumpAllocator;
use stage_1::bios_disk::BiosDisk;
use stage_1::bios_video::{BiosTextMode, _print};
use stage_1::config_parser::BootloaderConfig;
use stage_1::filesystem::FileSystem;
use stage_1::memory_map::get_memory_map;
use stage_1::unreal::{enter_stage2, enter_unreal_mode};
use stage_1::vesa::{BiosVesa, Res};

global_asm!(include_str!("init.s"));

#[no_mangle]
extern "C" fn bit16_entry(disk_number: u16) {
    enter_rust(disk_number);
}

static mut TEMP_ALLOC: MaybeUninit<SimpleBumpAllocator> = MaybeUninit::uninit();

fn enter_rust(disk_id: u16) {
    BiosTextMode::print_int_bytes(b"Quantum Bootloader (Stage1)\n");
    BiosTextMode::print_int_bytes(b"Unreal ...");
    unsafe {
        enter_unreal_mode();
    };
    BiosTextMode::print_int_bytes(b" OK\n");

    
    // TODO: Detect Memory First so we know if we have enough space, and where to put it.
    //       We should also look into reimplementing a lot of the 'BootInfo' Struct due
    //       to how it only gives a limited amount of info to the next stage. There is already
    //       fixme in the bootloader noting that we "don't" know the state of the vga buffer.
    unsafe {
        TEMP_ALLOC.write(SimpleBumpAllocator::new_from_ptr((0x00100000) as *mut u8, 0x03200000));
    }

    let temp_alloc = unsafe { TEMP_ALLOC.assume_init_mut() };

    let boot_info: &mut BootInfo = unsafe {
        &mut *(
            temp_alloc
                .allocate_region(size_of::<BootInfo>() + 0x10)
                .unwrap()
                .as_mut_ptr()
                as *mut BootInfo)
    };

    *boot_info = BootInfo::new();

    BiosTextMode::print_int_bytes(b"Loading files ");
    let fs =
        FileSystem::new(BiosDisk::new(disk_id as u8))
            .quarry_disk()
            .expect("Could not read any supported filesystems!")
            .mount_root_if_contains("/bootloader/bootloader.cfg")
            .expect("Could detect bootloader partition, please add \'/bootloader/bootloader.cfg\' to the bootloader filesystem for a proper boot!");

    let bootloader_config_file_ptr =
        temp_alloc.allocate_region(0x00100000 - (size_of::<BootInfo>() + 0x10)).unwrap();
    let bootloader_filename = "/bootloader/bootloader.cfg";

    fs.load_file_into_slice(bootloader_config_file_ptr, bootloader_filename)
        .expect("Unable to load bootloader config file!");

    BiosTextMode::print_int_bytes(b"...config...");
    let bootloader_config_string =
        unsafe { core::str::from_utf8_unchecked(bootloader_config_file_ptr) };

    let bootloader_config = BootloaderConfig::from_str(bootloader_config_string.trim())
        .expect("Unable to parse bootloader config!");

    let next_2_stage_bytes = fs
        .get_filesize_bytes(bootloader_config.get_stage2_file_path())
        .expect("Could not get stage2 filesize");

    let next_2_stage_ptr =
        temp_alloc
            .allocate_region(0x00100000)
            .unwrap();

    let next_3_stage_bytes = fs
        .get_filesize_bytes(bootloader_config.get_stage3_file_path())
        .expect("Could not get stage3 filesize");

    let next_3_stage_ptr =
        temp_alloc
            .allocate_region(next_3_stage_bytes + 0x10)
            .unwrap();

    let kernel_filesize_bytes = fs
        .get_filesize_bytes(bootloader_config.get_kernel_file_path())
        .expect("Could not get kernel filesize");

    let kernel_ptr =
        temp_alloc
            .allocate_region(kernel_filesize_bytes + 0x10)
            .unwrap();

    BiosTextMode::print_int_bytes(b"...stage2...");
    fs.load_file_into_slice(next_2_stage_ptr, bootloader_config.get_stage2_file_path())
        .expect("Could not load next stage!");

    BiosTextMode::print_int_bytes(b"...stage3...");
    fs.load_file_into_slice(next_3_stage_ptr, bootloader_config.get_stage3_file_path())
        .expect("Could not load next stage!");

    BiosTextMode::print_int_bytes(b"...kernel...");
    fs.load_file_into_slice(kernel_ptr, bootloader_config.get_kernel_file_path())
        .expect("Could not load kernel!");

    BiosTextMode::print_int_bytes(b" OK\n");

    boot_info.set_kernel_entry(
        BootMemoryDescriptor {
            ptr: kernel_ptr.as_ptr() as u64,
            size: kernel_filesize_bytes as u64,
        }
    );

    boot_info.set_stage_2_entry(
        BootMemoryDescriptor {
            ptr: next_2_stage_ptr.as_ptr() as u64,
            size: next_2_stage_bytes as u64,
        }
    );

    boot_info.set_stage_3_entry(
        BootMemoryDescriptor {
            ptr: next_3_stage_ptr.as_ptr() as u64,
            size: next_3_stage_bytes as u64,
        }
    );

    BiosTextMode::print_int_bytes(b"Getting memory map ... ");

    let amount_of_entries_allowed = 10;
    let memory_region_len = amount_of_entries_allowed * size_of::<E820Entry>();

    let memory_region =
        temp_alloc
            .allocate_region(memory_region_len + 0x10)
            .unwrap();

    let e820_ptr = memory_region as *mut [u8] as *mut E820Entry;
    let e820_ref = unsafe { core::slice::from_raw_parts_mut(e820_ptr, amount_of_entries_allowed) };

    get_memory_map(e820_ref);

    let mut amount_of_entries_found = 0;
    for entry in e820_ref {
        if entry.address > 0 || entry.len > 0 {
            amount_of_entries_found += 1;
        } else {
            break;
        }
    }

    boot_info.set_memory_map(e820_ptr, amount_of_entries_found);

    BiosTextMode::print_int_bytes(b"OK\n");

    BiosTextMode::print_int_bytes(b"Getting Vesa Info ... ");
    let raw_video_info = bootloader_config.get_recommended_video_info();
    let expected_res = Res {
        x: raw_video_info.0,
        y: raw_video_info.1,
        depth: 32,
    };

    let mut vesa = BiosVesa::new().quarry().unwrap();
    let closest_mode = vesa.find_closest_mode(expected_res).unwrap();
    BiosTextMode::print_int_bytes(b"OK\n");

    BiosTextMode::print_int_bytes(b"Setting Video Mode and jumping to stage2! ");
    vesa.set_mode(&closest_mode).unwrap();

    boot_info.set_video_information(VideoInformation {
        video_mode: 0,
        x: closest_mode.mode_data.width.into(),
        y: closest_mode.mode_data.height.into(),
        depth: closest_mode.mode_data.bpp.into(),
        framebuffer: closest_mode.mode_data.framebuffer,
    });


    unsafe {
        enter_stage2(
            next_2_stage_ptr.as_ptr(),
            boot_info as *const BootInfo as *const u8,
        );
    }
}

#[panic_handler]
#[allow(dead_code)]
fn panic(info: &PanicInfo) -> ! {
    BiosTextMode::print_int_bytes(b"Panic!!");
    _print(format_args!("{}", info));

    loop {}
}
