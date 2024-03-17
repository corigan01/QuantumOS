#![no_std]
#![no_main]

mod disk;
mod tiny_panic;

use core::{arch::global_asm, include_str};

global_asm!(include_str!("init.s"));

#[no_mangle]
extern "C" fn main(disk_number: u16) {
    let dap = disk::DiskAccessPacket::new(1, 0, 0x0800);
    dap.read(0x80);
    tiny_panic::fail(b'&');
}
