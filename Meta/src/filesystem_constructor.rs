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

use std::fs;
use std::fs::{create_dir_all, OpenOptions, remove_file};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::process::Command;
use anyhow::{anyhow, Context};
use fatfs::FatType::Fat16;
use fatfs::FormatVolumeOptions;
use mbrman::MBR;
use crate::artifacts::{get_program_path, get_target_directory, StageID};
use walkdir::WalkDir;
use crate::config_generator::BiosBootConfig;


pub fn make_and_construct_bios_image(kernel: &String, bios_stages: &Vec<(StageID, String)>) -> anyhow::Result<String> {
    // Make the formatted raw images first
    let ext2_img = make_ext2_fs(400 * 1024 * 1024)?;
    let fat_img = make_fat_fs(50 * 1024 * 1024)?;

    // Move artifacts into directories
    let target_path = get_target_directory()?;
    let bootloader_build_target_path = format!("{}/bootloader_build", target_path);
    let bootloader_sub_target_path = format!("{}/bootloader", bootloader_build_target_path);

    create_dir_all(&bootloader_build_target_path)?;
    create_dir_all(&bootloader_sub_target_path)?;

    let config_target_path = format!("{}/bootloader.cfg", bootloader_sub_target_path);

    make_bootloader_config_file(
        &config_target_path,
        16,
        (1280, 720),
        String::from("/kernel.elf"),
        format!("/bootloader/{}", StageID::Stage2.to_stage_id_string()),
        format!("/bootloader/{}", StageID::Stage3.to_stage_id_string())
    ).context(anyhow!("Could not generate bootloader config file"))?;

    // Copy the bootloader stages to the tmp folder
    for (stage_id, stage_path) in bios_stages {
        fs::copy(
            stage_path,
            format!("{}/{}", bootloader_sub_target_path, stage_id.to_stage_id_string())
        )?;
    }

    // Copy the kernel to the tmp folder
    fs::copy(
        kernel,
        format!("{}/kernel.elf", bootloader_build_target_path)
    )?;

    let stage1 = bios_stages.iter().find_map(|(stage_id, stage_path)| {
        if stage_id == &StageID::Stage1 {
            Some(stage_path)
        } else {
            None
        }
    }).ok_or(anyhow!("Could not get Stage1 path"))?;

    write_directory_into_fat_img(&bootloader_build_target_path, &fat_img)?;

    let disk_target_path = make_mbr_disk(&fat_img, &ext2_img)?;
    embed_stage1_into_img(&disk_target_path, stage1)?;

    remove_file(&fat_img)?;
    remove_file(&ext2_img)?;

    Ok(disk_target_path)
}

pub fn make_bootloader_config_file(
    file_location: &String,
    kernel_start_address_mb: usize,
    video_mode_preferred: (usize, usize),
    kernel_filepath: String,
    stage2_filepath: String,
    stage3_filepath: String
) -> anyhow::Result<()> {

    if Path::new(file_location).exists() {
        return Ok(());
    }

    let mut config_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(file_location)?;

    let config = BiosBootConfig {
        stage2_filepath,
        stage3_filepath,
        kernel_address: format!("{}", kernel_start_address_mb),
        kernel_filepath,
        video_mode_preferred,
    };

    config_file.write_fmt(format_args!("{}", config))?;
    config_file.sync_all()?;

    Ok(())
}

pub fn make_ext2_fs(file_size: usize) -> anyhow::Result<String> {
    let target_root = get_target_directory()?;
    let ext2_target_file = format!("{}/ext2.img", target_root);

    if Path::new(&ext2_target_file).exists() {
        remove_file(&ext2_target_file)?;
    }

    let ext2_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&ext2_target_file)?;

    ext2_file.set_len(file_size as u64)?;
    ext2_file.sync_all()?;

    let make_fs_path = get_program_path("mkfs.ext2")?;
    let command_run = Command::new(&make_fs_path)
        .arg(&ext2_target_file)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    if !command_run.success() {
        return Err(anyhow!("Failed to format {} with ext2", ext2_target_file));
    }

    Ok(ext2_target_file)
}

pub fn make_fat_fs(file_size: usize) -> anyhow::Result<String> {
    let target_root = get_target_directory()?;
    let fat_target_file = format!("{}/fat.img", target_root);

    if Path::new(&fat_target_file).exists() {
        remove_file(&fat_target_file)?;
    }

    let mut fat_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&fat_target_file)?;

    fat_file.set_len(file_size as u64)?;

    // Format the disk with fat
    let volume_id = *b"QBOOT      ";
    let fatfs_options = FormatVolumeOptions::new()
        .volume_label(volume_id)
        .drive_num(0x80)
        .fat_type(Fat16)
        .bytes_per_cluster(4096)
        .fats(2)
        .bytes_per_sector(512);

    fatfs::format_volume(&mut fat_file, fatfs_options)?;
    fat_file.sync_all()?;

    Ok(fat_target_file)
}

pub fn write_directory_into_fat_img(dir_path: &String, fat_img: &String) -> anyhow::Result<()> {
    let fat_img_open = OpenOptions::new()
        .read(true)
        .write(true)
        .open(fat_img)?;

    let fat_filesystem = fatfs::FileSystem::new(fat_img_open, fatfs::FsOptions::new())?;
    let fat_root_dir = fat_filesystem.root_dir();

    let mut root_chars_to_chop = 0;
    for entry in WalkDir::new(dir_path) {
        let entry = entry.context(anyhow!("Could not open DirEntry"))?;

        if root_chars_to_chop == 0 {
            root_chars_to_chop = entry.path().to_string_lossy().len();
            continue;
        }

        let system_path = entry.path();
        let quantum_path = &entry.path().to_string_lossy()[root_chars_to_chop..];

        if system_path.is_dir() {
            fat_root_dir.create_dir(quantum_path)?;
            continue;
        }

        let mut system_file = OpenOptions::new()
            .read(true)
            .open(system_path)?;

        let mut fat_path = fat_root_dir.create_file(quantum_path)?;

        let mut system_file_contents = Vec::new();
        system_file.read_to_end(&mut system_file_contents)?;
        fat_path.write_all(&system_file_contents)?;
    }

    Ok(())
}

pub fn embed_stage1_into_img(img: &String, stage1: &String) -> anyhow::Result<()> {
    let mut disk_file = OpenOptions::new()
        .write(true)
        .open(img)?;

    let mut stage1 = OpenOptions::new()
        .read(true)
        .open(stage1)?;


    let mut stage1_contents = Vec::new();
    stage1.seek(SeekFrom::Start(0))?;
    stage1.read_to_end(&mut stage1_contents)?;

    disk_file.seek(SeekFrom::Start(0))?;
    disk_file.write_all(&stage1_contents[..410])?;
    disk_file.seek(SeekFrom::Start(512))?;
    disk_file.write_all(&stage1_contents[512..])?;

    disk_file.sync_all()?;

    Ok(())
}

pub fn make_mbr_disk(fat_img: &String, ext2_img: &String) -> anyhow::Result<String> {
    let target_dir = get_target_directory()?;
    let mbr_target_path = format!("{}/disk.img", target_dir);

    if Path::new(&mbr_target_path).exists() {
        remove_file(&mbr_target_path)?;
    }

    let mut disk_image = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&mbr_target_path)?;

    let mut fat_image = OpenOptions::new()
        .read(true)
        .write(true)
        .open(fat_img)?;

    let mut ext2_image = OpenOptions::new()
        .read(true)
        .write(true)
        .open(ext2_img)?;

    let fat_size = fat_image.metadata()?.len();
    let ext2_size = ext2_image.metadata()?.len();

    let fat_size_sectors = (fat_size / 512) as u32;
    let ext2_size_sectors = (ext2_size / 512) as u32;

    disk_image.set_len(fat_size + ext2_size + (1024 * 1024))?;

    let mut mbr = MBR::new_from(&mut disk_image, 512, [12, 51, 12, 43])?;

    let optimal_fat_start = mbr.find_optimal_place(fat_size_sectors)
        .ok_or(anyhow!("Could not find optimal place for fat partition"))?;

    mbr[1] = mbrman::MBRPartitionEntry {
        boot: mbrman::BOOT_ACTIVE,
        first_chs: mbrman::CHS::empty(),
        sys: 0x83,
        last_chs: mbrman::CHS::empty(),
        starting_lba: optimal_fat_start,
        sectors: fat_size_sectors,
    };

    let optimal_ext2_start = mbr.find_optimal_place(ext2_size_sectors)
        .ok_or(anyhow!("Could not find optimal place for ext2 partition"))?;

    mbr[2] = mbrman::MBRPartitionEntry {
        boot: mbrman::BOOT_INACTIVE,
        first_chs: mbrman::CHS::empty(),
        sys: 0x83,
        last_chs: mbrman::CHS::empty(),
        starting_lba: optimal_ext2_start,
        sectors: ext2_size_sectors
    };

    mbr[3] = mbrman::MBRPartitionEntry::empty();
    mbr[4] = mbrman::MBRPartitionEntry::empty();

    mbr.write_into(&mut disk_image)?;
    disk_image.sync_all()?;

    let fat_start_disk_offset = optimal_fat_start as u64 * 512;
    let ext2_start_disk_offset = optimal_ext2_start as u64 * 512;

    // Write fat image onto disk
    disk_image.seek(SeekFrom::Start(fat_start_disk_offset))?;
    fat_image.seek(SeekFrom::Start(0))?;

    let mut fat_image_buffer = vec![0_u8; 512];
    loop {
        match fat_image.read(&mut fat_image_buffer) {
            Ok(0) => break,
            Ok(n) => {
                disk_image.write_all(&fat_image_buffer[0..n])?;
            }
            Err(e) => return Err(e.into())
        }
    }

    // Write ext2 image onto disk
    disk_image.seek(SeekFrom::Start(ext2_start_disk_offset))?;
    ext2_image.seek(SeekFrom::Start(0))?;

    let mut ext2_image_buffer = vec![0_u8; 512];
    loop {
        match ext2_image.read(&mut ext2_image_buffer) {
            Ok(0) => break,
            Ok(n) => {
                disk_image.write_all(&ext2_image_buffer[0..n])?;
            }
            Err(e) => return Err(e.into())
        }
    }

    disk_image.sync_all()?;

    Ok(mbr_target_path)
}