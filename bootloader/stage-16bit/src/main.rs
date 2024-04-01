#![no_std]
#![no_main]

use crate::{disk::BiosDisk, fatfs::Fat, mbr::Mbr};
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
    let disk = BiosDisk::new(disk_id);
    let mbr = Mbr::new(disk).unwrap();
    let partition = mbr.partition(1).unwrap();

    let fat = Fat::new(partition).unwrap();

    bios_println!("{:#?}", fat);
}
