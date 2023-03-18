/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
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

use core::ptr::slice_from_raw_parts;
use core::str;

use crate::error::BootloaderError;
use crate::filesystem::{DiskMedia, ValidFilesystem};
use crate::filesystem::fat::bios_parameter_block::BiosBlock;
use crate::filesystem::fat::fat_file::FatFile;
use crate::filesystem::partition::PartitionEntry;

pub mod bios_parameter_block;
pub mod fat16;
pub mod fat_file;

pub trait FatValid {
    fn is_valid(bpb: &BiosBlock) -> bool;
}

#[derive(Copy, Clone, Debug)]
pub enum FatType {
    Fat32,
    Fat16,
    Fat12,

    Unknown
}

impl FatType {
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Unknown => false,

            _ => true
        }
    }
}

pub struct Fatfs<'a, DiskType: DiskMedia> {
    disk: &'a DiskType,
    partition: &'a PartitionEntry,
    bpb: BiosBlock
}

impl<'a, DiskType: DiskMedia + 'a> Fatfs<'a, DiskType> {
    fn get_sector_offset(disk: &DiskType, partition: &PartitionEntry, sector: usize) -> Result<[u8; 512], BootloaderError> {
        if let Some(partition) = partition.get_start_sector() {
            return disk.read(partition + sector);
        }

        Err(BootloaderError::NoValid)
    }

    pub fn quarry(disk: &DiskType, partition: &PartitionEntry) -> Result<FatType, BootloaderError> {
        let sector_data = Self::get_sector_offset(disk, partition, 0)?;
        let bpb = BiosBlock::new(sector_data);

        Ok(bpb.get_fat_type())
    }

    pub fn new(disk: &'a DiskType, partition: &'a PartitionEntry) -> Result<Self, BootloaderError> {
        let fat_type = Self::quarry(&disk, &partition)?;

        if !fat_type.is_valid() {
            return Err(BootloaderError::NoValid);
        }

        let boot_sector_data = disk.read(0)?;

        Ok(Self {
            disk,
            partition,
            bpb: BiosBlock::new(boot_sector_data),
        })
    }

    pub fn get_children_file_within_parent(&self, parent: &FatFile, filename: &str) -> Result<FatFile, BootloaderError> {

    }

    pub fn contains_file(&self, filename: &str) -> Result<bool, BootloaderError> {
        let mut parent = self.bpb.get_root_file_entry()?;
        let (_, mut file_consumption_part) = filename.clone().split_at(1);

        loop {
            if let Some(next_char_i) = file_consumption_part.find('/') {
                let (child_name, remaining) = file_consumption_part.split_at(next_char_i);

                file_consumption_part = remaining;

                parent = self.get_children_file_within_parent(&parent, child_name)?;
            } else {
                let final_file = self.get_children_file_within_parent(&parent, file_consumption_part);

                if let Ok(file) = &final_file {
                    return Ok(true);
                }

                final_file?
            }
        }
    }

}

impl<'a, DiskType: DiskMedia + 'a> ValidFilesystem<DiskType> for Fatfs<'a, DiskType> {
    fn is_valid(disk: &DiskType, partition: &PartitionEntry) -> bool {
        if let Ok(fat_type) = Self::quarry(disk, partition) {
            fat_type.is_valid()
        } else {
            false
        }
    }

    fn does_contain_file(disk: &DiskType, partition: &PartitionEntry, filename: &str) -> Result<bool, BootloaderError> {
        let fatfs = Fatfs::<DiskType>::new(disk, partition)?;

        fatfs.contains_file(filename)
    }
}