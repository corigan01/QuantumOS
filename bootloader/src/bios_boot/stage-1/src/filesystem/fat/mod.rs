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

#[cfg(debug)]
use crate::{bios_print, bios_println};

use core::str;

use quantum_lib::heapless_string::HeaplessString;

use crate::filesystem::fat::bios_parameter_block::{BiosBlock, FatTableEntryType};
use crate::filesystem::fat::fat_file::{FatDirectoryEntry, FatFile, FatFileType, FatLongFileName};
use crate::filesystem::partition::PartitionEntry;
use crate::filesystem::{DiskMedia, ValidFilesystem};
use bootloader::error::BootloaderError;

pub mod bios_parameter_block;
pub mod fat16;
pub mod fat_file;

pub trait FatValid {
    fn is_valid(bpb: &BiosBlock) -> bool;
}

#[derive(Copy, Clone)]
pub enum FatType {
    Fat32,
    Fat16,
    Fat12,

    Unknown,
}

impl FatType {
    pub fn is_valid(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

pub struct Fatfs<'a, DiskType: DiskMedia> {
    disk: &'a DiskType,
    partition: &'a PartitionEntry,
    bpb: BiosBlock,
}

impl<'a, DiskType: DiskMedia + 'a> Fatfs<'a, DiskType> {
    fn get_sector_offset(
        disk: &DiskType,
        partition: &PartitionEntry,
        sector: usize,
    ) -> Result<[u8; 512], BootloaderError> {
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
        if let Some(partition_start) = partition.get_start_sector() {
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
        } else {
            Err(BootloaderError::NoValid)
        }
    }

    fn get_fat_entry(&mut self, cluster_id: usize) -> Result<usize, BootloaderError> {
        let fat_entry_size = self.bpb.get_file_allocation_table_entry_size_bytes()?;

        let inter_index = cluster_id % (512 / fat_entry_size);
        let sector_offset = cluster_id / (512 / fat_entry_size);

        let first_fat_sector = self.bpb.file_allocation_table_begin()?;
        let real_fat_sector = first_fat_sector + sector_offset;

        let fat_data = Self::get_sector_offset(self.disk, self.partition, real_fat_sector)?;

        let entry = self
            .bpb
            .get_file_allocation_table_entry(inter_index, &fat_data)?;

        if entry == 0 {
            return Err(BootloaderError::OutOfBounds);
        }

        Ok(entry)
    }

    fn run_on_all_entries_in_dir_and_return_on_true<Function>(
        &mut self,
        fat_file: &FatFile,
        run_on_each: Function,
    ) -> Result<FatFile, BootloaderError>
    where
        Function: Fn(&FatFile) -> bool,
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
        // all are read, but for now we should be able to boot with just a few entries in
        // each entry.
        let mut long_file_name_tmp_buffer = [0_u8; 256];
        for i in 0..16 {
            let file_entry = unsafe { &*(data.as_ptr().add(32 * i) as *const FatDirectoryEntry) };

            let mut file = file_entry.to_fat_file()?;

            if file.filetype == FatFileType::LongFileName {
                let long_file_name =
                    unsafe { &*(file_entry as *const FatDirectoryEntry as *const FatLongFileName) };

                unsafe {
                    long_file_name.accumulate_name(&mut long_file_name_tmp_buffer);
                };

                continue;
            } else {
                // Make the filename
                let mut total_chars = 0;
                for (i, value) in long_file_name_tmp_buffer.iter().enumerate() {
                    if *value == 0 {
                        total_chars = i;
                        break;
                    }
                }

                // set the filename
                let (new_slice, _) = long_file_name_tmp_buffer.split_at(total_chars);

                file.filename = HeaplessString::from_bytes(new_slice).unwrap();
            }

            if file.filetype != FatFileType::Zero {
                if run_on_each(&file) {
                    return Ok(file);
                }

                long_file_name_tmp_buffer.fill(0);
            }
        }

        Err(BootloaderError::NoValid)
    }

    pub fn get_children_file_within_parent(
        &mut self,
        parent: &FatFile,
        filename: &str,
    ) -> Result<FatFile, BootloaderError> {
        if filename.is_empty() {
            #[cfg(debug)]
            bios_println!(
                "Finding children with filename \"\" does not make sense, maybe an error?"
            );
            return Err(BootloaderError::NotSupported);
        }

        let looking_cstring = HeaplessString::from_str(filename).unwrap();

        self.run_on_all_entries_in_dir_and_return_on_true(parent, |entry| {
            entry.filename == looking_cstring
        })
    }

    pub fn contains_file(&mut self, filename: &str) -> Result<FatFile, BootloaderError> {
        let mut parent = self.bpb.get_root_file_entry()?;
        let (_, mut file_consumption_part) = filename.split_at(1);

        loop {
            return if let Some(next_char_i) = file_consumption_part.find('/') {
                let child_name = if next_char_i == 0 {
                    let (_, child_name) = file_consumption_part.split_at(1);

                    child_name
                } else {
                    let (child_name, remaining) = file_consumption_part.split_at(next_char_i);

                    file_consumption_part = remaining;

                    parent = self.get_children_file_within_parent(&parent, child_name)?;

                    continue;
                };

                self.get_children_file_within_parent(&parent, child_name)
            } else {
                self.get_children_file_within_parent(&parent, file_consumption_part)
            };
        }
    }

    unsafe fn follow_clusters_and_load_into_buffer(
        &mut self,
        file: &FatFile,
        ptr: *mut u8,
    ) -> Result<(), BootloaderError> {
        let mut following_cluster = file.start_cluster;
        let mut ptr_offset = 0;

        // FIXME: This is kinda sloppy code, maybe fix in the future
        loop {
            let cluster_id = following_cluster - 2;

            let first_data_sector = self.bpb.data_cluster_begin()?;
            let sector_offset = (cluster_id * self.bpb.cluster_size()?) + first_data_sector;
            let cluster_size = self.bpb.cluster_size()?;

            for e in 0..cluster_size {
                let partition_offset = self.partition.get_start_sector();

                if partition_offset.is_none() {
                    return Err(BootloaderError::NoValid);
                }

                let partition_offset = partition_offset.unwrap_unchecked();
                let total_sector_offset = partition_offset + sector_offset + e;

                // Sometimes the bios is unable to load a file at some memory addresses,
                // so we have some basic error handling here that looks if a DiskIOError
                // has occurred and will attempt to read the disk again, but this time
                // at a buffer that is significantly smaller and at lower memory.
                let moved_ptr = ptr.add(ptr_offset);

                let disk_read_status: Result<(), BootloaderError> = if moved_ptr as u32 > 64 * 1024
                {
                    Err(BootloaderError::DiskIOError)
                } else {
                    self.disk.read_ptr(total_sector_offset, moved_ptr)
                };

                match disk_read_status {
                    Ok(_) => {
                        ptr_offset += 512;
                    }

                    Err(BootloaderError::DiskIOError) => {
                        let read_data =
                            Self::get_sector_offset(self.disk, self.partition, sector_offset + e)?;

                        // FIXME: This makes the loading slow, but the bios is having trouble accessing high
                        // memory, so unfortunately this is our only option for moving memory
                        for i in &read_data {
                            *ptr.add(ptr_offset) = *i;
                            ptr_offset += 1;
                        }
                    }

                    _ => {
                        return Err(BootloaderError::DiskIOError);
                    }
                }
            }

            let table_value = self.get_fat_entry(following_cluster)?;
            let next_cluster_type = self.bpb.convert_usize_to_fat_table_entry(table_value)?;

            match next_cluster_type {
                FatTableEntryType::Valid(cluster) => {
                    following_cluster = cluster;
                }
                FatTableEntryType::EndOfFile => {
                    return Ok(());
                }

                _ => {
                    #[cfg(debug)]
                    bios_println!("READ ERROR ------- {}", table_value);

                    return Err(BootloaderError::NoValid);
                }
            }
        }
    }

    pub unsafe fn load_file_at_ptr(
        &mut self,
        filename: &str,
        ptr: *mut u8,
    ) -> Result<(), BootloaderError> {
        let file = self.contains_file(filename)?;

        self.follow_clusters_and_load_into_buffer(&file, ptr)
    }

    pub fn get_vol_label(&self) -> Result<[u8; 11], BootloaderError> {
        let bpb_label = self.bpb.get_vol_label()?;

        Ok(bpb_label)
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
    fn get_volume_name(
        disk: &DiskType,
        partition: &PartitionEntry,
    ) -> Result<[u8; 11], BootloaderError> {
        let fatfs = Fatfs::<DiskType>::new(disk, partition)?;

        fatfs.get_vol_label()
    }

    fn does_contain_file(
        disk: &DiskType,
        partition: &PartitionEntry,
        filename: &str,
    ) -> Result<bool, BootloaderError> {
        let mut fatfs = Fatfs::<DiskType>::new(disk, partition)?;

        Ok(fatfs.contains_file(filename).is_ok())
    }

    fn size_of_file(
        disk: &DiskType,
        partition: &PartitionEntry,
        filename: &str,
    ) -> Result<usize, BootloaderError> {
        let mut fatfs = Fatfs::<DiskType>::new(disk, partition)?;

        let file = fatfs.contains_file(filename)?;

        Ok(file.filesize_bytes)
    }

    unsafe fn load_file_to_ptr(
        disk: &DiskType,
        partition: &PartitionEntry,
        filename: &str,
        ptr: *mut u8,
    ) -> Result<(), BootloaderError> {
        let mut fatfs = Fatfs::<DiskType>::new(disk, partition)?;

        fatfs.load_file_at_ptr(filename, ptr)
    }
}
