/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
    Part of the Quantum OS Project

Copyright 2024 Gavin Kellam

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

#![no_std]
#![no_main]

mod disk;
mod partition;
mod tiny_panic;

use bios::video;
use core::{arch::global_asm, include_str};

global_asm!(include_str!("init.s"));

#[no_mangle]
extern "C" fn main(disk_number: u16) {
    let bootable = unsafe { &mut *partition::find_bootable() };
    let mut load_ptr = 0x7E00;

    loop {
        let load_count = bootable.count.min(32) as u16;
        disk::DiskAccessPacket::new(load_count, bootable.lba as u64, load_ptr).read(disk_number);

        bootable.lba += load_count as u32;
        bootable.count -= load_count as u32;
        load_ptr += 512 * load_count as u32;

        if bootable.count == 0 {
            break;
        }
    }

    video::putc(b'O');
    video::putc(b'K');

    unsafe {
        let stage1: fn(u16) = core::mem::transmute(0x7E00_usize);
        stage1(disk_number);
    };

    tiny_panic::fail(b'&');
}
