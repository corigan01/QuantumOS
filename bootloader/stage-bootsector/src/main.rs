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

    video::putc(b's');

    unsafe {
        core::arch::asm!("
            mov ax, 0x7C00
            mov sp, ax
            push {disk_number:x}
            ", disk_number = in(reg) disk_number);
        core::arch::asm!("ljmp $0x00, $0x7e00", options(att_syntax));
    }

    tiny_panic::fail(b'&');
}
