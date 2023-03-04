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

use core::panic::PanicInfo;
use core::arch::{global_asm};

use stage_1::bios_disk::BiosDisk;
use stage_1::{bios_print, bios_println};
use stage_1::bios_ints::{BiosInt, TextModeColor};
use stage_1::fat::{Extended32, FAT};
use stage_1::mbr::{MasterBootRecord, PartitionEntry};
use stage_1::vesa::BasicVesaInfo;

global_asm!(include_str!("init.s"));

#[no_mangle]
extern "C" fn bit16_entry(disk_number: u16) {
    enter_rust(disk_number);
    panic!("Stage should not return!");
}


fn enter_rust(disk: u16) {
    bios_println!("\n --- Quantum Boot loader 16 ---\n");

    let mbr = unsafe { MasterBootRecord::read_from_disk(disk as u8) };

    bios_println!("Found {} partitions on boot disk {:x}!", mbr.total_valid_partitions(), disk);

    if let Some(entry_id) = mbr.get_bootable_partition() {
        bios_println!("Partition {:?} is bootable and has partition type of {:x?}",
            entry_id, mbr.get_partition_entry(entry_id).get_partition_type());

    } else {
        bios_println!(" | Could not find valid partition!");
        panic!("No bootable partitions found, I dont know how we even booted!");
    }

    let fat = FAT::new_from_disk(disk as u8)
        .expect("No valid bootable partitions with FAT32 found!");

    bios_println!("Detected {:?} type on disk {:x} -- \'{}\' ",
        fat.get_fat_type(),
        disk,
        fat.get_disk_label().unwrap()
    );

    loop {};
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    bios_println!("{}", info);

    loop {}
}