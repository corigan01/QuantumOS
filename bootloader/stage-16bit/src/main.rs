#![no_std]
#![no_main]

use crate::{disk::BiosDisk, fatfs::Fat, mbr::Mbr};
use io::{Read, Seek};
use unreal::enter_unreal;

mod console;
mod disk;
mod error;
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
    let disk = BiosDisk::new(disk_id);
    let mbr = Mbr::new(disk).unwrap();
    let partition = mbr.partition(1).unwrap();

    let mut fat = Fat::new(partition).unwrap();
    let mut fat_file = fat.open("/qconfig.cfg").unwrap();

    let mut buffer = [0u8; 32];
    fat_file.seek(0);
    fat_file.read(&mut buffer).unwrap();

    bios_print!(
        "FILE: --------\n{}\n---------\n{:x?}",
        core::str::from_utf8(&buffer).unwrap(),
        buffer
    );

    bios_println!("{:#?}", fat);
}
