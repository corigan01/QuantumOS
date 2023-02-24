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
use core::arch::{asm, global_asm};

use stage_1::bios_disk::BiosDisk;
use stage_1::bios_println;
use stage_1::vesa::BasicVesaInfo;


global_asm!(include_str!("init.s"));


#[no_mangle]
extern "C" fn bit16_entry(disk_number: u16) {

    enter_rust(disk_number);
    panic!("Stage should not return!");
}


fn enter_rust(disk: u16) {
    let boot_disk = BiosDisk::new(disk as u8);

    bios_println!("\nVBE INFO = {:#?}", BasicVesaInfo::new());
    bios_println!("DiskID = 0x{:X}", disk);

    unsafe { boot_disk.read_from_disk(0x7c00 as *mut u8, 0..1); }

    bios_println!("Read boot-sector 0x{:02X}", unsafe { *(0x7c00 as *mut u128) } );


    loop {};
}




#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    bios_println!("PANIC: {:#?}", info);



    loop {}
}