#![no_main]
#![no_std]

use core::panic::PanicInfo;

fn main() {}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
