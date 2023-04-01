/*
  ____                 __
 / __ \__ _____ ____  / /___ ____ _
/ /_/ / // / _ `/ _ \/ __/ // /  ' \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/
  Part of the Quantum OS Project

Copyright 2023 Gavin Kellam

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

use fatfs::{FileSystem, FormatVolumeOptions, FsOptions};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::Command;
use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cargo = std::env::var("CARGO").unwrap_or("cargo".into());

    let current_dir = env::current_dir()?;

    let target_dir =
        std::env::var("OUT_DIR").unwrap_or(format!("{}/target", current_dir.display()));
    let target_dir = format!("{}", target_dir);

    let stage1 = Command::new(cargo)
        .current_dir("bootloader/src/bios_boot/stage-1")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("i386-quantum_loader.json")
        .arg(format!("--target-dir={}", target_dir))
        .stdout(std::process::Stdio::piped())
        .status();

    let stage1_path = format!(
        "{}/target/i386-quantum_loader/release/stage-1",
        current_dir.display()
    );

    let stage1_to_bin = Command::new("objcopy")
        .arg("-I")
        .arg("elf32-i386")
        .arg("-O")
        .arg("binary")
        .arg(&stage1_path)
        .stdout(std::process::Stdio::piped())
        .status();

    let mut stage1_file = std::fs::OpenOptions::new()
        .read(true)
        .open(&stage1_path)
        .unwrap();
    let mut stage1_entire_file: Vec<u8> = Vec::new();

    stage1_file.seek(SeekFrom::Start(0)).unwrap();
    stage1_file.read_to_end(&mut stage1_entire_file).unwrap();

    println!("{:?} {:?}", stage1, stage1_to_bin);

    {
        println!("Making fat image!");
        let mut fat_img = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fatfs.img")?;

        fat_img.set_len(40 * 1024 * 1024)?;

        let volume_id = *b"BOOTLOADER ";

        {
            let fatfs_options = FormatVolumeOptions::new()
                .volume_label(volume_id)
                .drive_num(0x80);

            fatfs::format_volume(&mut fat_img, fatfs_options)?;

            let fat = FileSystem::new(&mut fat_img, FsOptions::new())?;
            let root_dir = fat.root_dir();

            root_dir.create_dir("bootloader")?;

            // TODO: Replace with real kernel
            root_dir
                .create_file("kernel.elf")?
                .write_all(&[0xfa; 4096])?;

            let bootloader_dir = root_dir.open_dir("bootloader")?;
            bootloader_dir
                .create_file("bootloader.cfg")?
                .write_all(&*b"KERNEL=/kernel.elf")?;
        }

        fat_img.sync_all()?;

        println!("Making disk image!");

        let mut disk_img = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/disk.img")?;

        disk_img.set_len(512 * 1024 * 1024)?;

        let mut mbr = mbrman::MBR::new_from(&mut disk_img, 512, [12, 51, 12, 43])?;

        let fat_size = 50 * 1024 * 1024;
        let sector_size = fat_size / 512;
        let first_sector_of_fat = mbr.find_optimal_place(sector_size).unwrap();

        mbr[1] = mbrman::MBRPartitionEntry {
            boot: mbrman::BOOT_ACTIVE,
            first_chs: mbrman::CHS::empty(),
            sys: 0x83,
            last_chs: mbrman::CHS::empty(),
            starting_lba: first_sector_of_fat,
            sectors: sector_size,
        };

        let next_partition_size = mbr.get_maximum_partition_size()?;
        let next_starting_sector = mbr.find_optimal_place(next_partition_size).unwrap();

        mbr[2] = mbrman::MBRPartitionEntry {
            boot: mbrman::BOOT_INACTIVE,
            first_chs: mbrman::CHS::empty(),
            sys: 0x83,
            last_chs: mbrman::CHS::empty(),
            starting_lba: next_starting_sector,
            sectors: next_partition_size,
        };

        mbr[3] = mbrman::MBRPartitionEntry::empty();
        mbr[4] = mbrman::MBRPartitionEntry::empty();

        mbr.write_into(&mut disk_img)?;
        disk_img.sync_all()?;

        println!("Done formatting disk!");

        for (i, p) in mbr.iter() {
            if p.is_used() {
                println!(
                    "Partition #{}: type = {:?}, size = {}MiB, starting lba = {}",
                    i,
                    p.sys,
                    (p.sectors * mbr.sector_size) / (1024 * 1024),
                    p.starting_lba
                );
            }
        }

        println!("Copying fatfs into disk.img");

        fat_img.seek(SeekFrom::Start(0))?;
        disk_img.seek(SeekFrom::Start(
            (first_sector_of_fat * mbr.sector_size) as u64,
        ))?;

        let mut buffer = vec![0; mbr.sector_size as usize];
        loop {
            match fat_img.read(&mut buffer) {
                Ok(0) => {
                    println!("Zero!");
                    break;
                }
                Ok(n) => disk_img.write_all(&buffer[0..n])?,
                Err(e) => return Err(e.into()),
            }
        }

        disk_img.seek(SeekFrom::Start(0))?;
        disk_img.write(&mut stage1_entire_file[0..410])?;
        disk_img.seek(SeekFrom::Start(512))?;
        disk_img.write(&mut stage1_entire_file[512..])?;

        println!("Syncing disk img");

        disk_img.sync_all()?;
    }

    fs::remove_file("target/fatfs.img")?;

    //        COMMAND qemu-system-i386 -d cpu_reset --no-shutdown -drive format=raw,file=${CMAKE_BINARY_DIR}/disk.img

    let qemu = Command::new("qemu-system-i386")
        .arg("-d")
        .arg("cpu_reset")
        .arg("--no-shutdown")
        .arg("-drive")
        .arg(format!("format=raw,file={}", "target/disk.img"))
        .stdout(std::process::Stdio::piped())
        .status();

    println!("Done! {:?}", qemu);

    Ok(())
}
