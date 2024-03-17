#![no_std]
#![no_main]

mod tiny_panic;
use core::{arch::global_asm, include_str};

global_asm!(include_str!("init.s"));

#[no_mangle]
extern "C" fn main(_disk_number: u16) {
    tiny_panic::fail(b'm');
}
