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

use fatfs::FatType::Fat16;
use fatfs::{FileSystem, FormatVolumeOptions, FsOptions};
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::process::Command;
use owo_colors::OwoColorize;

pub fn delete_disk_img(disk_name: String) -> Result<(), Box<dyn std::error::Error>> {
    fs::remove_file(disk_name)?;

    Ok(())
}

pub fn create_fat_img_from_directory(
    fat_img_path: &String,
    directory_path: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let fat_img_path = format!("{}/fat.img", fat_img_path);

    // Make the raw file
    let mut fat_img = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&fat_img_path)?;

    fat_img.set_len(40 * 1024 * 1024)?;

    // Format the disk with fat
    let volume_id = *b"QBOOT      ";
    let fatfs_options = FormatVolumeOptions::new()
        .volume_label(volume_id)
        .drive_num(0x80)
        .fat_type(Fat16)
        .bytes_per_cluster(4096)
        .fats(2)
        .bytes_per_sector(512);

    fatfs::format_volume(&mut fat_img, fatfs_options)?;

    // Open the fat img
    let fat = FileSystem::new(&mut fat_img, FsOptions::new())?;
    let fat_root_dir = fat.root_dir();

    // Read all the files and directories in `directory_path`
    let directory = fs::read_dir(directory_path)?;

    // FIXME: Should be a function todo this because there is a lot of redundant code
    #[allow(clippy::manual_flatten)]
    for items in directory {
        if let Ok(item) = items {
            if item.file_type().unwrap().is_file() {
                let file_path = item.path();
                let filename = file_path.file_name().unwrap().to_str().unwrap();

                let mut fat_equivalent = fat_root_dir.create_file(filename)?;
                let mut opened_file = OpenOptions::new().read(true).open(file_path)?;

                let mut opened_file_data = Vec::new();
                opened_file.read_to_end(&mut opened_file_data)?;

                fat_equivalent.write_all(opened_file_data.as_slice())?;
            } else if item.file_type().unwrap().is_dir() {
                let dir_path = item.path();
                let item_filename = item.file_name();
                let dir_name = item_filename.to_str().unwrap();

                let fat_equivalent = fat_root_dir.create_dir(dir_name)?;
                let opened_dir = fs::read_dir(dir_path)?;

                for files in opened_dir {
                    if let Ok(file) = files {
                        if file.file_type().unwrap().is_file() {
                            let file_path = file.path();
                            let filename = file_path.file_name().unwrap().to_str().unwrap();

                            let mut fat_equivalent = fat_equivalent.create_file(filename)?;
                            let mut opened_file = OpenOptions::new().read(true).open(file_path)?;

                            let mut opened_file_data = Vec::new();
                            opened_file.read_to_end(&mut opened_file_data)?;

                            fat_equivalent.write_all(opened_file_data.as_slice())?;
                        }
                    }
                }
            }
        }
    }

    Ok(fat_img_path)
}

pub fn make_ext2_fs(
    ext2_img_path_root: &Path,
    directory_path: &Path,
    image_size: usize
) -> Result<Path, Box<dyn std::error::Error>> {
    let ext2_img = format!("{}/ext2.img", ext2_img_path_root.display());

    {
        let image = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&ext2_img)?;

        image.set_len(image_size as u64)?;
        image.sync_all()?;
    }

    let status = Command::new("mkfs.ext2")
        .arg(ext2_img)
        .status()
        .expect("Unable to run 'mkfs.ext2' which is required to format ext2 partition");

    if !status.success() {
        panic!("Could not generate ext2fs partition");
    }


    Ok(*Path::new(&ext2_img))
}

pub fn make_mbr_disk(
    path: &String,
    fat_path: &String,
    stage_1_path: &String,
) -> Result<String, Box<dyn std::error::Error>> {
    let disk_img_path = format!("{}/disk.img", path);

    let mut disk_img = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&disk_img_path)?;

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

    println!("     {} Done formatting disk!", "Quantum".green().bold());

    for (i, p) in mbr.iter() {
        if p.is_used() {
            println!(
                "     {} Partition #{}: type = {:?}, size = {}MiB, starting lba = {}",
                "Quantum".green().bold(),
                i,
                p.sys,
                (p.sectors * mbr.sector_size) / (1024 * 1024),
                p.starting_lba
            );
        }
    }

    let mut fat_img = OpenOptions::new().read(true).write(true).open(fat_path)?;

    fat_img.seek(SeekFrom::Start(0))?;
    disk_img.seek(SeekFrom::Start(
        (first_sector_of_fat * mbr.sector_size) as u64,
    ))?;

    let mut buffer = vec![0; mbr.sector_size as usize];
    loop {
        match fat_img.read(&mut buffer) {
            Ok(0) => {
                break;
            }
            Ok(n) => disk_img.write_all(&buffer[0..n])?,
            Err(e) => return Err(e.into()),
        }
    }

    let mut stage1_file = OpenOptions::new().read(true).open(&stage_1_path).unwrap();
    let mut stage1_entire_file: Vec<u8> = Vec::new();

    stage1_file.seek(SeekFrom::Start(0)).unwrap();
    stage1_file.read_to_end(&mut stage1_entire_file).unwrap();

    disk_img.seek(SeekFrom::Start(0))?;
    disk_img.write_all(&stage1_entire_file[0..410])?;
    disk_img.seek(SeekFrom::Start(512))?;
    disk_img.write_all(&stage1_entire_file[512..])?;

    disk_img.sync_all()?;

    Ok(disk_img_path)
}
