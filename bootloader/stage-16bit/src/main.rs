#![no_std]
#![no_main]

use core::fmt::Write;

use console::bios_write_char;
use unreal::enter_unreal;

mod console;
mod disk;
mod panic;
mod unreal;

#[no_mangle]
#[link_section = ".begin"]
extern "C" fn entry(disk_id: u16) {
    unsafe { enter_unreal() };

    bios_println!();
    main(disk_id);
    loop {}
}

fn main(disk_id: u16) {
    bios_println!("Qauntum Loader");
}
