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

use crate::bios_disk::BiosDisk;
use crate::cstring::{CStringOwned, CStringRef};
use crate::fat::bpb::BiosParametersBlock;
use crate::fat::fat_16::Extended16;
use crate::fat::fat_32::Extended32;
use crate::mbr::{MasterBootRecord, PartitionEntry};
use crate::{bios_print, bios_println};
use core::mem::size_of;

pub mod bpb;
pub mod fat_16;
pub mod fat_32;

pub trait FatExtCluster {
    fn is_valid_sig(&self) -> bool;
    fn get_vol_string(&self) -> Option<CStringRef>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FatType {
    Fat12,
    Fat16,
    Fat32,

    NotImplemented,
    NotFat,
}

#[derive(Debug)]
pub enum FileType {
    ReadOnly,
    Hidden,
    System,
    LongFileName,
    VolumeLabel,
    Directory,
    File,
    Unknown,
}

impl FileType {
    pub fn new_from_file(entry: &DirectoryEntry) -> Self {
        let attr = entry.file_attributes;

        match attr {
            0x01 => Self::ReadOnly,
            0x02 => Self::Hidden,
            0x04 => Self::System,
            0x08 => Self::VolumeLabel,
            0x0f => Self::LongFileName,
            0x10 => Self::Directory,
            0x20 => Self::File,

            _ => Self::Unknown,
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct DirectoryEntry {
    pub file_name: [u8; 8],
    pub file_extension: [u8; 3],
    pub file_attributes: u8,
    reserved_win_nt: u8,
    pub creation_time_tens_of_second: u8,
    pub creation_time: u16,
    pub creation_date: u16,
    pub last_accessed_byte: u16,
    pub high_entry_bytes: u16,
    pub modification_time: u16,
    pub modification_date: u16,
    pub low_entry_bytes: u16,
    pub file_bytes: u32,
}

impl FatType {
    pub fn is_valid(&self) -> bool {
        *self != Self::NotImplemented && *self != Self::NotFat
    }
}

pub struct FAT {
    disk: BiosDisk,
    sector_info: PartitionEntry,
    fat_type: Option<FatType>,
    bpb: Option<BiosParametersBlock>,
}

impl FAT {
    fn populate_bpb(&mut self) {
        let mut sector_temp_buffer = [0_u8; 512];
        let head_sector = self.sector_info.get_sector_start();

        unsafe {
            self.disk.read_from_disk(
                &mut sector_temp_buffer as *mut u8,
                head_sector..(head_sector + 1),
            );
        }

        self.bpb =
            Some(unsafe { *(&mut sector_temp_buffer as *mut u8 as *mut BiosParametersBlock) });
    }

    pub fn get_fat_type(&self) -> FatType {
        self.fat_type.unwrap_or(FatType::NotFat)
    }

    pub fn get_root_cluster_number(&self) -> Option<usize> {
        Some(match self.fat_type? {
            FatType::Fat32 => {
                unsafe { self.bpb?.get_ext_bpb::<Extended32>()? }.root_cluster_number as usize
            }

            // The docs suggest that the root dir should always be after the FAT, so we are just
            // hard coding it here.
            // FIXME: Implement a better system for finding and validating that this is root
            FatType::Fat16 | FatType::Fat12 => 2,

            _ => return None,
        })
    }

    pub fn get_first_fat_sector(&self) -> Option<usize> {
        Some((self.bpb?.reserved_sectors as usize))
    }

    pub fn get_fat_size(&self) -> Option<usize> {
        Some(match self.fat_type? {
            FatType::Fat32 => {
                unsafe { self.bpb?.get_ext_bpb::<Extended32>()? }.sectors_per_fat as usize
            }
            FatType::Fat16 | FatType::Fat12 => self.bpb?.sectors_per_fat as usize,

            _ => return None,
        })
    }

    pub fn get_total_sectors(&self) -> Option<usize> {
        if self.bpb?.low_sectors == 0 || self.bpb?.low_sectors == u16::MAX {
            Some(self.bpb?.high_sectors as usize)
        } else {
            Some(self.bpb?.low_sectors as usize)
        }
    }

    pub fn get_data_sector_count(&self) -> Option<usize> {
        let table_sectors = (self.bpb?.reserved_sectors as usize * self.get_fat_size()?)
            + self.get_root_dir_sector_count()?;
        let s_count = self.bpb?.reserved_sectors as usize + table_sectors;

        Some(self.get_total_sectors()? - s_count)
    }

    pub fn get_root_dir_sector_count(&self) -> Option<usize> {
        Some(
            (((self.bpb?.root_entries * 32) + (self.bpb?.bytes_per_sector - 1))
                / (self.bpb?.bytes_per_sector)) as usize,
        )
    }

    pub fn get_first_data_sector(&self) -> Option<usize> {
        let number_of_fat = self.bpb?.num_of_fats as usize;
        let fat_size = self.get_fat_size()?;
        let root_sectors = self.get_root_dir_sector_count()?;

        Some(
            (number_of_fat * fat_size)
                + (self.bpb?.reserved_sectors as usize)
                + self.sector_info.get_sector_start()
                + root_sectors,
        )
    }

    // FIXME: This is really just a hack, fix to include values of all fat sizes
    pub unsafe fn get_fat_table<T>(&self, index: usize) -> Option<T>
    where
        T: Clone,
    {
        let bytes_to_grab = size_of::<T>();
        let table_offset = (index * bytes_to_grab) % 512;
        let mut table_sector = (index * bytes_to_grab) / 512;

        let sector =
            table_sector + self.get_first_fat_sector()? + self.sector_info.get_sector_start();

        let mut sector_tmp = [0u8; 512];
        unsafe {
            self.disk
                .read_from_disk(sector_tmp.as_mut_ptr(), sector..(sector + 1));
        };

        let bytes = unsafe { &*(sector_tmp.as_ptr().add(table_offset) as *const T) };

        Some(bytes.clone())
    }

    pub fn get_first_sector_of_cluster(&self, cluster: usize) -> Option<usize> {
        Some((self.bpb?.sectors_per_cluster as usize * cluster) + self.get_first_data_sector()?)
    }

    pub fn get_root_dir_sector(&self) -> Option<usize> {
        Some(self.get_first_data_sector()? - self.get_root_dir_sector_count()?)
    }

    fn read_data_cluster(&self, cluster: usize) -> Option<[u8; 512]> {
        let mut tmp_sector_data = [0u8; 512];
        let sector_to_read = self.get_first_sector_of_cluster(cluster)?;

        unsafe {
            self.disk.read_from_disk(
                tmp_sector_data.as_mut_ptr(),
                sector_to_read..(sector_to_read + 1),
            );
        }

        Some(tmp_sector_data)
    }


    pub fn print_root_entries(&self) -> Option<()> {
        let root_sector = self.get_root_dir_sector()?;
        let mut sector_tmp = [0u8; 512];

        unsafe {
            self.disk
                .read_from_disk(sector_tmp.as_mut_ptr(), root_sector..(root_sector + 1));
        };

        let entries = self.bpb?.root_entries;
        bios_println!("Entries {}", entries);

        for i in 0..(512 / 32) {
            let dir_entry = unsafe { &*(sector_tmp.as_ptr().add(i * 32) as *const DirectoryEntry) };

            if dir_entry.modification_date == 0 {
                continue;
            }

            let entry_id = (dir_entry.low_entry_bytes as usize)
                + if self.get_fat_type() != FatType::Fat12 && self.get_fat_type() != FatType::Fat16
                {
                    (dir_entry.high_entry_bytes as usize) << 16
                } else {
                    0
                };

            let fat_table = unsafe { self.get_fat_table::<u16>(entry_id)? } as usize;
            let mut cluster_data = self.read_data_cluster(entry_id - 2)?;

            bios_println!(
                "({}): {}{}{} -- {:?}(0x{:02x}) [C: {}, F: {:04x}] => {}",
                i,
                CStringRef::from_bytes(&dir_entry.file_name),
                if dir_entry.file_extension[0] != 0 && dir_entry.file_extension[0] != 0x20 {
                    '.'
                } else {
                    ' '
                },
                CStringRef::from_bytes(&dir_entry.file_extension),
                FileType::new_from_file(&dir_entry),
                dir_entry.file_attributes,
                entry_id,
                fat_table,
                unsafe { CStringRef::from_ptr(cluster_data.as_mut_ptr()) }
            );
        }

        None
    }

    pub fn validate_fat(&self) -> FatType {
        if let Some(bpb) = &self.bpb {
            // check if we understand this bpb (checking if this partition is fat)
            if !bpb.validate_fat() {
                return FatType::NotFat;
            }

            let total_clusters =
                self.get_total_sectors().unwrap_or(0) / bpb.sectors_per_cluster as usize;

            // Now check the fat type
            if total_clusters < 4085 && bpb.check_ext_bpb::<Extended16>() {
                return FatType::Fat12;
            } else if bpb.check_ext_bpb::<Extended16>() {
                return FatType::Fat16;
            } else if bpb.check_ext_bpb::<Extended32>() {
                return FatType::Fat32;
            }

            return FatType::NotImplemented;
        }

        FatType::NotFat
    }

    pub fn get_vol_label(&self) -> Option<CStringOwned> {
        let bpb_ref = &self.bpb?;

        let vol_label = match self.fat_type? {
            FatType::Fat32 => &unsafe { bpb_ref.get_ext_bpb::<Extended32>()? }.vol_label,
            FatType::Fat16 | FatType::Fat12 => {
                &unsafe { bpb_ref.get_ext_bpb::<Extended16>()? }.vol_label
            }

            _ => return None,
        };

        Some(unsafe { CStringOwned::from_ptr(vol_label as *const u8, vol_label.len()) })
    }

    pub fn new_from_disk(disk_id: u8) -> Option<Self> {
        let mbr = unsafe { MasterBootRecord::read_from_disk(disk_id) };
        let partition = mbr.get_partition_entry(
            mbr.get_bootable_partition()
                .expect("No bootable partitions found!"),
        );

        let mut data = Self {
            disk: BiosDisk::new(disk_id),
            sector_info: *partition,
            fat_type: None,
            bpb: None,
        };

        data.populate_bpb();

        let fat_type = data.validate_fat();
        data.fat_type = Some(fat_type);

        // Finally return
        if fat_type.is_valid() {
            Some(data)
        } else {
            None
        }
    }
}
