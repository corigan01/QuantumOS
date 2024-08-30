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
