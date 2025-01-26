/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
    Part of the Quantum OS Project

Copyright 2024 Gavin Kellam

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

#![no_std]
#![no_main]

use crate::{disk::BiosDisk, mbr::Mbr};
use bios::memory::MemoryEntry;
use bios::video::Vesa;
use bootloader::Stage16toStage32;
use bump_alloc::BumpAlloc;
use config::BootloaderConfig;
use fs::fatfs::Fat;
use fs::io::Read;
use lldebug::make_debug;
use lldebug::{debug_ready, logln};
use serial::Serial;
use unreal::enter_unreal;

mod bump_alloc;
mod config;
mod disk;
mod mbr;
mod memory;
mod panic;
mod unreal;

make_debug! {
    "Serial": Option<Serial> = Serial::probe_first(serial::baud::SerialBaud::Baud115200);
}

#[no_mangle]
#[link_section = ".begin"]
extern "C" fn entry(disk_id: u16) {
    unsafe { enter_unreal() };

    logln!();
    main(disk_id);
}

#[debug_ready]
fn main(disk_id: u16) -> ! {
    logln!("Quantum Loader");

    // - Memory Setup
    let memory_map = crate::memory::memory_map();

    let ideal_region = memory_map
        .iter()
        .find(|region| {
            region.region_type == MemoryEntry::REGION_FREE
                && region.base_address >= (1024 * 1024)
                && region.region_length >= (1024 * 1024 * 16)
        })
        .expect("Cannot find high memory above 1MB!");

    let mut alloc =
        unsafe { BumpAlloc::new(ideal_region.base_address, ideal_region.region_length) };

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

    let vesa = Vesa::quarry().ok();

    // - Stage-to-Stage
    alloc.align_ptr_to(align_of::<Stage16toStage32>());

    let stage_to_stage = unsafe {
        &mut *(alloc
            .allocate(size_of::<Stage16toStage32>())
            .expect("Unable to allocate Stage-to-Stage!")
            .as_mut_ptr() as *mut Stage16toStage32)
    };

    unsafe {
        core::ptr::copy_nonoverlapping(
            memory_map.as_ptr(),
            stage_to_stage.memory_map.as_mut_ptr(),
            memory_map.len().min(stage_to_stage.memory_map.len()),
        )
    };

    if let Some((closest_video_id, closest_video_info)) = vesa
        .and_then(|vesa| {
            vesa.modes()
                .filter_map(|id| id.querry().ok().map(|mode| (id, mode)))
                .filter(|(_, mode)| mode.bpp == 32)
                .reduce(|closest_mode, (id, mode)| {
                    if closest_mode.1.width.abs_diff(want_x) > mode.width.abs_diff(want_x)
                        && closest_mode.1.height.abs_diff(want_y) > mode.height.abs_diff(want_y)
                    {
                        (id, mode)
                    } else {
                        closest_mode
                    }
                })
        })
        .and_then(|(video_id, video_info)| video_id.set().ok().map(|_| (video_id, video_info)))
    {
        stage_to_stage.video_mode = Some((closest_video_id, closest_video_info));

        logln!(
            "Optimal Video Mode id={:#04x}: {}x{} {}bbp",
            closest_video_id.get_id(),
            closest_video_info.width,
            closest_video_info.height,
            closest_video_info.bpp
        );
    } else {
        stage_to_stage.video_mode = None;
        logln!("Video mode failed!");
    }

    // - Bootloader32
    let mut bootloader32 = fatfs
        .open(qconfig.bootloader32)
        .expect("Unable to find bootloader32");

    // Our bootloader needs to be at 0x00200000
    let bootloader32_entrypoint = 0x00200000 as *mut u8;
    alloc.push_ptr_to(bootloader32_entrypoint);

    logln!(
        "Loading stage32 '{}' ({} Bytes)",
        qconfig.bootloader32,
        bootloader32.filesize()
    );
    let bootloader32_buffer = unsafe { alloc.allocate(bootloader32.filesize()) }.unwrap();
    bootloader32
        .read(bootloader32_buffer)
        .expect("Unable to read bootloader32");

    // - Bootloader64
    let mut bootloader64 = fatfs
        .open(qconfig.bootloader64)
        .expect("Unable to find bootloader64");

    // Our bootloader needs to be at 0x00400000
    let bootloader64_entrypoint = 0x00400000 as *mut u8;
    alloc.push_ptr_to(bootloader64_entrypoint);

    logln!(
        "Loading stage64 '{}' ({} Bytes)",
        qconfig.bootloader64,
        bootloader64.filesize()
    );
    let bootloader64_buffer = unsafe { alloc.allocate(bootloader64.filesize()) }.unwrap();
    bootloader64
        .read(bootloader64_buffer)
        .expect("Unable to read bootloader64");

    // kernel elf file
    let kernel_offset = 0x00500000 as *mut u8;
    alloc.push_ptr_to(kernel_offset);

    let mut kernel_file = fatfs.open(qconfig.kernel).expect("Unable to find kernel");

    logln!(
        "Loading kernel '{}' ({} Bytes)",
        qconfig.kernel,
        kernel_file.filesize()
    );
    let kernel_buffer = unsafe { alloc.allocate(kernel_file.filesize()) }.unwrap();
    kernel_file
        .read(kernel_buffer)
        .expect("Unable to read kernel");

    let stack_region = unsafe { alloc.allocate(1024 * 1024) }.unwrap();

    // The initfs needs to be 2Mib page aligned
    alloc.align_ptr_to(1024 * 1024);

    // Initfs region
    let mut initfs_file = fatfs
        .open(qconfig.initfs)
        .expect("Unable to load initfs region");

    logln!(
        "Loading initfs '{}' ({} Bytes)",
        qconfig.initfs,
        initfs_file.filesize()
    );
    let initfs_buffer = unsafe { alloc.allocate(initfs_file.filesize()) }.unwrap();
    initfs_file
        .read(initfs_buffer)
        .expect("Unable to read initfs");

    stage_to_stage.bootloader_stack_ptr = (stack_region.as_ptr() as u64, 1024 * 1024);
    stage_to_stage.stage32_ptr = (
        bootloader32_entrypoint as u64,
        bootloader32_buffer.len() as u64,
    );
    stage_to_stage.stage64_ptr = (
        bootloader64_entrypoint as u64,
        bootloader64_buffer.len() as u64,
    );
    stage_to_stage.kernel_ptr = (kernel_buffer.as_ptr() as u64, kernel_buffer.len() as u64);
    stage_to_stage.initfs_ptr = (initfs_buffer.as_ptr() as u64, initfs_buffer.len() as u64);

    unsafe {
        unreal::enter_stage2(
            bootloader32_entrypoint,
            stack_region.as_ptr().add(1024 * 1024),
            stage_to_stage as *const Stage16toStage32,
        )
    };
}
