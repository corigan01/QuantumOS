#![no_std]
#![no_main]

mod disk;
mod partition;
mod tiny_panic;

use core::{arch::global_asm, include_str};

global_asm!(include_str!("init.s"));

#[no_mangle]
extern "C" fn main(disk_number: u16) {
    let bootable = unsafe { &mut *partition::find_bootable() };

    loop {
        disk::DiskAccessPacket::new(1, bootable.lba as u64, 0x7E00).read(disk_number);

        bootable.lba += 1;
        bootable.count -= 1;

        if bootable.count <= 0 {
            break;
        }
    }

    tiny_panic::fail(b'&');
}
