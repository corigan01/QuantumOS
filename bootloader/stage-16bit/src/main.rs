#![no_std]
#![no_main]

use crate::{disk::BiosDisk, mbr::Mbr};
use bios::memory::MemoryEntry;
use bios::video::Vesa;
use bump_alloc::BumpAlloc;
use config::BootloaderConfig;
use fs::fatfs::Fat;
use fs::io::Read;
use unreal::enter_unreal;

mod bump_alloc;
mod config;
mod console;
mod disk;
mod mbr;
mod memory;
mod panic;
mod unreal;
// mod vbe;

#[no_mangle]
#[link_section = ".begin"]
extern "C" fn entry(disk_id: u16) {
    unsafe { enter_unreal() };

    bios_println!();
    main(disk_id);
}

fn main(disk_id: u16) -> ! {
    bios_println!("Qauntum Loader");

    // - Memory Setup
    let memory_map = crate::memory::memory_map();

    let ideal_region = memory_map
        .iter()
        .find(|region| {
            region.region_type != MemoryEntry::REGION_RESERVED
                && region.base_address >= (1024 * 1024)
        })
        .expect("Cannot find high memory above 1MB!");

    let mut alloc = unsafe {
        BumpAlloc::new(
            ideal_region.base_address as *mut u8,
            ideal_region.region_length as usize,
        )
    };

    // - Filesystem Enumeration

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
            fat.entry_of("bootloader/qconfig.cfg")
                .ok()
                .map(|_| part_number)
        })
        .expect("Cannot find valid FAT Partition!");

    let mut fatfs = Fat::new(mbr.partition(partition_number).unwrap()).unwrap();

    // - Config File
    let mut qconfig = fatfs.open("bootloader/qconfig.cfg").unwrap();
    let qconfig_filesize = qconfig.filesize();
    let qconfig_buffer = unsafe { alloc.allocate(qconfig_filesize) }.unwrap();
    qconfig
        .read(qconfig_buffer)
        .expect("Unable to read qconfig!");

    let qconfig = core::str::from_utf8(&qconfig_buffer).unwrap();
    let qconfig = BootloaderConfig::parse_file(&qconfig).unwrap();

    // - Video Mode Config
    let (want_x, want_y) = qconfig.expected_vbe_mode.unwrap_or((800, 600));

    let vesa = Vesa::quarry().unwrap();
    let modes = vesa
        .modes()
        .filter_map(|id| id.querry().ok().map(|mode| (id, mode)))
        .reduce(|closest_mode, (id, mode)| {
            if closest_mode.1.width.abs_diff(want_x) > mode.width.abs_diff(want_x)
                && closest_mode.1.height.abs_diff(want_y) > mode.height.abs_diff(want_y)
            {
                return (id, mode);
            }

            closest_mode
        });
    bios_println!("Closest = {:?}", modes);

    // - Bootloader32
    let mut bootloader32 = fatfs
        .open(qconfig.bootloader32)
        .expect("Unable to find bootloader32");
    let bootloader32_buffer = unsafe { alloc.allocate(bootloader32.filesize()) }.unwrap();
    bootloader32
        .read(bootloader32_buffer)
        .expect("Unable to read bootloader32");

    panic!("Not supposed to return!");
}
