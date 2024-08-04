#![no_std]
#![no_main]

use crate::{disk::BiosDisk, mbr::Mbr};
use bios::memory::MemoryEntry;
use fs::fatfs::Fat;
use fs::io::{Read, Seek, SeekFrom};
use unreal::enter_unreal;

mod bump_alloc;
mod console;
mod disk;
mod error;
mod mbr;
mod memory;
mod panic;
mod unreal;

#[no_mangle]
#[link_section = ".begin"]
extern "C" fn entry(disk_id: u16) {
    unsafe { enter_unreal() };

    bios_println!();
    main(disk_id);
}

fn main(disk_id: u16) -> ! {
    bios_println!("Qauntum Loader");
    let memory_map = crate::memory::memory_map();

    let ideal_region = memory_map
        .iter()
        .find(|region| {
            region.region_type != MemoryEntry::REGION_RESERVED
                && region.base_address >= (1024 * 1024)
        })
        .expect("Cannot find high memory above 1MB!");

    // FIXME: We need to figure out a new way of handing partitions from mbr
    //        since partitions currently cannot be used to create Fats that
    //        escape this closure. This means we need to create a new Fat
    //        which should be avoided if its already known to be valid.
    let mut mbr = Mbr::new(BiosDisk::new(disk_id)).expect("Cannot read MBR!");
    let partition_number = (0..4)
        .into_iter()
        .find_map(|part_number| {
            let Some(partition) = mbr.partition(part_number) else {
                return None;
            };

            let mut fat = Fat::new(partition).ok()?;
            fat.entry_of("qconfig.cfg").ok().map(|_| part_number)
        })
        .expect("Cannot find valid FAT Partition!");

    let mut fatfs = Fat::new(mbr.partition(partition_number).unwrap()).unwrap();
    let qconfig = fatfs.open("qconfig.cfg").unwrap();

    panic!("Not supposed to return!");
}
