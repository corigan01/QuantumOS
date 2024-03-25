#![no_std]
#![no_main]

use crate::{
    disk::BiosDisk,
    io::{Read, Seek},
};
use unreal::enter_unreal;

mod console;
mod disk;
mod fatfs;
mod io;
mod mbr;
mod panic;
mod unreal;

#[no_mangle]
#[link_section = ".begin"]
extern "C" fn entry(disk_id: u16) {
    unsafe { enter_unreal() };

    bios_println!();
    main(disk_id);
    panic!("Not supposed to return!");
}

fn main(disk_id: u16) {
    bios_println!("Qauntum Loader");
    let mut disk = BiosDisk::new(disk_id);
    disk.seek(511);
    let mut buffer = [0; 520];

    bios_println!("Reading Disk...");
    disk.read(&mut buffer);
    bios_println!("Done: {:x?}", buffer);
}
