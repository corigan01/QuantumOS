#![no_std]
#![no_main]

use unreal::enter_unreal;

mod panic;
mod unreal;

#[no_mangle]
#[link_section = ".begin"]
extern "C" fn entry(disk_id: u16) {
    unsafe { enter_unreal() };

    for c in b"test" {
        panic::putc(*c);
    }
    // panic!("Test");

    loop {}
}
