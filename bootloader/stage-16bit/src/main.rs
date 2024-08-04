#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use crate::{disk::BiosDisk, mbr::Mbr};
use unreal::enter_unreal;

use fs::fatfs::Fat;
use fs::io::{Read, Seek, SeekFrom};

mod bump_alloc;
mod console;
mod disk;
mod error;
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

#[no_mangle]
#[link_name = ".memory_map"]
static mut MEMORY_MAP_AREA: MaybeUninit<[bios::memory::MemoryEntry; 32]> = MaybeUninit::zeroed();

fn main(disk_id: u16) {
    bios_println!("Qauntum Loader");

    // let disk = BiosDisk::new(disk_id);
    // let mbr = Mbr::new(disk).unwrap();
    // let partition = mbr.partition(1).unwrap();

    // let mut fat = Fat::new(partition).unwrap();
    // let mut fat_file = fat.open("/qconfig.cfg").unwrap();

    // let mut buffer = [0u8; 32];
    // fat_file.seek(SeekFrom::Start(0));
    // fat_file.read(&mut buffer).unwrap();

    // bios_print!(
    //     "FILE: --------\n{}\n---------\n{:x?}",
    //     core::str::from_utf8(&buffer).unwrap(),
    //     buffer
    // );
    // bios_println!("{:#?}", fat);

    let stable_regions =
        unsafe { bios::memory::read_mapping(MEMORY_MAP_AREA.assume_init_mut()) }.unwrap();
    let memory_regions = unsafe { &MEMORY_MAP_AREA.assume_init_mut()[..stable_regions] };

    bios_println!("stable regions: {}", stable_regions);

    for region in memory_regions {
        bios_println!(
            "[{:04b}] - {:08x} : {}",
            region.region_type,
            region.base_address,
            region.region_length
        );
    }
}
