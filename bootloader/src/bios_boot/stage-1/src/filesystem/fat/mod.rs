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
use crate::{bios_print, bios_println};
use crate::cstring::{CStringOwned, CStringRef};

use crate::error::BootloaderError;
use crate::filesystem::{DiskMedia, ValidFilesystem};
use crate::filesystem::fat::bios_parameter_block::{BiosBlock, FatTableEntryType};
use crate::filesystem::fat::fat_file::{FatDirectoryEntry, FatFile, FatFileType, FatLongFileName};
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
        let partition_start = partition.get_start_sector().unwrap_or(0);
        let boot_sector_data = disk.read(partition_start)?;

        let bpb = BiosBlock::new(boot_sector_data);

        if !bpb.extended_type.is_valid() {
            return Err(BootloaderError::NoValid);
        }

        Ok(Self {
            disk,
            partition,
            bpb,
        })
    }

    fn get_fat_entry(&self, cluster_id: usize) -> Result<usize, BootloaderError> {
        let fat_entry_size = self.bpb.get_file_allocation_table_entry_size_bytes()?;

        let inter_index = cluster_id % (512 / fat_entry_size);
        let sector_offset = cluster_id / (512 / fat_entry_size);

        let first_fat_sector = self.bpb.file_allocation_table_begin()?;
        let real_fat_sector = first_fat_sector + sector_offset;

        let data = Self::get_sector_offset(self.disk, self.partition, real_fat_sector)?;

        self.bpb.get_file_allocation_table_entry(inter_index, &data)
    }

    fn run_on_all_entries_in_dir_and_return_on_true<Function>(&self, fat_file: &FatFile, run_on_each: Function )
        -> Result<FatFile, BootloaderError>
        where Function: Fn(&FatFile) -> bool
    {

        let data = if fat_file.filetype != FatFileType::Root {
            let cluster_id = fat_file.start_cluster - 2;

            let first_data_sector = self.bpb.data_cluster_begin()?;
            let sector_offset = (cluster_id * self.bpb.cluster_size()?) + first_data_sector;

            Self::get_sector_offset(self.disk, self.partition, sector_offset)?
        } else {
            let root_cluster = self.bpb.root_cluster_begin()?;

            Self::get_sector_offset(self.disk, self.partition, root_cluster)?
        };

        // FIXME: If a dir has more then 16 entries we need to trace and add them and loop until
        // all are read, but for now we should be able to boot with just a few entires in
        // each entry.
        let mut long_file_name_tmp_buffer = [0_u8; 512];
        for i in 0..16 {
          let file_entry = unsafe {
            &*(data.as_ptr().add(32 * i) as *const FatDirectoryEntry)
          };

            let mut file = file_entry.to_fat_file();

            if file.filetype != FatFileType::Unknown {
                let fat_entry = self.get_fat_entry(file.start_cluster)?;
            }


            if file.filetype == FatFileType::LongFileName {
                let long_file_name = unsafe {
                    &*(file_entry as *const FatDirectoryEntry as *const FatLongFileName)
                };

                unsafe {
                    long_file_name.accumulate_name(&mut long_file_name_tmp_buffer);
                };

                continue;

            } else {
                // Make the filename
                let mut total_chars = 0;
                for i in 0..long_file_name_tmp_buffer.len() {
                    let value = long_file_name_tmp_buffer[i];

                    if value == 0 {
                        total_chars = i;
                        break;
                    }
                }

                // set the filename
                file.filename = unsafe {
                    CStringOwned::from_ptr(
                        long_file_name_tmp_buffer.as_ptr(),
                        total_chars)
                };
            }

            if file.filetype != FatFileType::Unknown {
                if run_on_each(&file) {
                    return Ok(file);
                }

                long_file_name_tmp_buffer.fill(0);
            }
        }

        Err(BootloaderError::NoValid)
    }


    pub fn get_children_file_within_parent(&self, parent: &FatFile, filename: &str) -> Result<FatFile, BootloaderError> {
        if filename.len() == 0 {
            bios_println!("[W]: Finding children with filename \"\" does not make sense, maybe an error?");
            return Err(BootloaderError::NotSupported)
        }

        let looking_cstring = unsafe {
            CStringOwned::from_ptr(filename.as_ptr(), filename.len())
        };

        self.run_on_all_entries_in_dir_and_return_on_true(&parent,
            |entry| {

                entry.filename == looking_cstring
            })
    }

    pub fn contains_file(&self, filename: &str) -> Result<FatFile, BootloaderError> {
        let mut parent = self.bpb.get_root_file_entry()?;
        let (_, mut file_consumption_part) = filename.split_at(1);

        loop {
            if let Some(next_char_i) = file_consumption_part.find('/') {

                let child_name = if next_char_i == 0 {
                    let (_, child_name) = file_consumption_part.split_at(1);


                    child_name
                } else {
                    let (child_name, remaining) = file_consumption_part.split_at(next_char_i);

                    file_consumption_part = remaining.clone();

                    parent = self.get_children_file_within_parent(&parent, child_name)?;

                    continue;
                };

                return self.get_children_file_within_parent(&parent, child_name);
            }
        }
    }

    unsafe fn follow_clusters_and_load_into_buffer(&self, file: &FatFile, ptr: *mut u8) -> Result<(), BootloaderError> {
        let mut following_cluster = file.start_cluster;
        let mut ptr_offset = 0;

        // FIXME: This is kinda sloppy code, maybe fix in the future
        loop {
            let cluster_id = following_cluster - 2;

            let first_data_sector = self.bpb.data_cluster_begin()?;
            let sector_offset = (cluster_id * self.bpb.cluster_size()?) + first_data_sector;

            let read_data = Self::get_sector_offset(self.disk, self.partition, sector_offset)?;

            for i in read_data {
                *ptr.add(ptr_offset) = i;
                ptr_offset += 1;
            }

            let next_cluster_type =
                self.bpb.convert_usize_to_fat_table_entry(
                    self.get_fat_entry(following_cluster)?
                )?;

            match next_cluster_type {
                FatTableEntryType::Valid(cluster) => {
                    following_cluster = cluster;
                }
                FatTableEntryType::EndOfFile => {
                    return Ok(());
                }

                _ => return Err(BootloaderError::NoValid)
            }

        }
    }

    pub unsafe fn load_file_at_location(&self, filename: &str, ptr: *mut u8) -> Result<(), BootloaderError> {
        let file = self.contains_file(filename)?;

        self.follow_clusters_and_load_into_buffer(&file, ptr)
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

        Ok(fatfs.contains_file(filename).is_ok())
    }

    unsafe fn load_file_to_ptr(disk: &DiskType, partition: &PartitionEntry, filename: &str, ptr: *mut u8) -> Result<(), BootloaderError> {
        let fatfs = Fatfs::<DiskType>::new(disk, partition)?;

        fatfs.load_file_at_location(filename, ptr)
    }
}