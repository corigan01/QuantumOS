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
    let mut load_ptr = 0x7E00;

    loop {
        disk::DiskAccessPacket::new(1, bootable.lba as u64, load_ptr).read(disk_number);

        bootable.lba += 1;
        bootable.count -= 1;
        load_ptr += 0x200;

        if bootable.count == 0 {
            break;
        }
    }

    tiny_panic::putc(b's');

    unsafe {
        core::arch::asm!("
            and esp, 0xffffff00
            push {disk_number:x}
            ", disk_number = in(reg) disk_number);
        core::arch::asm!("ljmp $0x00, $0x7e00", options(att_syntax));
    }

    tiny_panic::fail(b'&');
}
