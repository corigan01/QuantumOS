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

use bootloader::boot_info::{BootInfo, SimpleRamFs};
use bootloader::BootMemoryDescriptor;
use quantum_lib::simple_allocator::SimpleBumpAllocator;

use stage_1::bios_disk::BiosDisk;
use stage_1::bios_ints::{BiosInt, TextModeColor};
use stage_1::bios_println;
use stage_1::bootloader_stack_information::{get_stack_used_bytes, get_total_stack_size};
use stage_1::config_parser::BootloaderConfig;
use stage_1::filesystem::FileSystem;
use stage_1::memory_detection::MemoryMap;
use stage_1::vesa::{BiosVesa, Res};

global_asm!(include_str!("init.s"));

#[no_mangle]
extern "C" fn bit16_entry(disk_number: u16) {
    bios_println!("\n --- Quantum Boot loader 16 ---\n");
    enter_rust(disk_number);
    panic!("Stage1 should not return!");
}

static mut TEMP_ALLOC: Option<SimpleBumpAllocator> = None;

fn enter_rust(disk_id: u16) {
    let mut boot_info = BootInfo::default();

    unsafe {
        TEMP_ALLOC = SimpleBumpAllocator::new_from_ptr(0x00100000 as *mut u8, 0x03200000);
    }

    let bootloader_config;

    {
        let fs =
            FileSystem::new(BiosDisk::new(disk_id as u8))
                .toggle_logging()
                .quarry_disk()
                .expect("Could not read any supported filesystems!")
                .mount_root_if_contains("/bootloader/bootloader.cfg")
                .expect("Could detect bootloader partition, please add \'/bootloader/bootloader.cfg\' to the bootloader filesystem for a proper boot!");

        let bootloader_config_file_ptr =
            unsafe { TEMP_ALLOC.as_mut().unwrap().allocate_region(256).unwrap() };
        let bootloader_filename = "/bootloader/bootloader.cfg";

        fs.load_file_into_slice(bootloader_config_file_ptr, bootloader_filename)
            .expect("Unable to load bootloader config file!");

        bootloader_config =
            BootloaderConfig::from_str(core::str::from_utf8(bootloader_config_file_ptr).unwrap())
                .expect("Unable to parse bootloader config!");

        bios_println!("{:#?}", bootloader_config);

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

        fs.load_file_into_slice(next_stage_ptr, bootloader_config.get_stage2_file_path())
            .expect("Could not load next stage!");

        fs.load_file_into_slice(kernel_ptr, bootloader_config.get_kernel_file_path())
            .expect("Could not load kernel!");

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

        bios_println!(
            "stack ({} / {})",
            get_stack_used_bytes(),
            get_total_stack_size()
        );
    }

    {
        let bootloader_video_mode = bootloader_config.get_recommended_video_info();

        let mut vga = BiosVesa::new()
            .quarry()
            .expect("Could not quarry Vesa information");

        let mode = vga
            .find_closest_mode(Res {
                x: bootloader_video_mode.0,
                y: bootloader_video_mode.1,
                depth: 24,
            })
            .expect("Could not find closest mode");

        bios_println!("Mode info {:#?}", &mode.get_res());

        //vga.set_mode(mode).unwrap();
        //vga.clear_display();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    bios_println!("{}", info);

    loop {}
}
