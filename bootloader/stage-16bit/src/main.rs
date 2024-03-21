#![no_std]
#![no_main]

use crate::{disk::BiosDisk, io::Read};
use unreal::enter_unreal;

mod console;
mod disk;
mod io;
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
    let mut disk = BiosDisk::new(disk_id);
    let mut buffer = [0; 512];

    disk.read(&mut buffer);
}
