#![no_std]
#![no_main]

use core::fmt::Write;

use unreal::enter_unreal;

mod panic;
mod unreal;

#[no_mangle]
#[link_section = ".begin"]
extern "C" fn entry(_disk_id: u16) {
    unsafe { enter_unreal() };

    panic::BiosPrinter::write_fmt(
        &mut panic::BiosPrinter {},
        format_args!("This is a test {}", unsafe { *(0x10_000 as *const u8) }),
    )
    .unwrap();

    loop {}
}
