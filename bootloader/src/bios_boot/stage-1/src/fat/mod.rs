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
use crate::{bios_print, bios_println};
use crate::cstring::{CStringRef, CStringOwned};
use crate::fat::bpb::BiosParametersBlock;
use crate::fat::fat_32::{DirectoryEntry32, Extended32};
use crate::mbr::{MasterBootRecord, PartitionEntry};

pub mod fat_32;
pub mod bpb;

pub trait FatExtCluster{
    fn is_valid_sig(&self) -> bool;
    fn get_vol_string(&self) -> Option<CStringRef>;
}

#[derive(Copy, Clone, Debug)]
pub enum FatType {
    Fat16,
    Fat32,
    ExFat,
    VFat,

    NotImplemented,
    NotFat
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
        let head_sector = self.sector_info.get_sector_start() as u16;

        unsafe {
            self.disk.read_from_disk(
                &mut sector_temp_buffer as *mut u8,
                head_sector..(head_sector + 1)
            );
        }

        self.bpb = Some(unsafe {
            *(&mut sector_temp_buffer as *mut u8 as *mut BiosParametersBlock)
        });
    }

    pub fn get_fat_type(&self) -> FatType {
        self.fat_type.unwrap_or(FatType::NotFat)
    }

    pub fn get_root_cluster_number(&self) -> Option<usize> {
        Some(match self.fat_type? {
                FatType::Fat32 => unsafe { self.bpb?.get_ext_bpb::<Extended32>()? }.root_cluster_number as usize,
                _ => return None
        })
    }

    pub fn get_first_fat_sector(&self) -> Option<usize> {
        Some(
            (self.bpb?.reserved_sectors as usize) + (self.sector_info.get_sector_start() as usize)
        )
    }

    pub fn get_fat_size(&self) -> Option<usize> {
        match self.fat_type? {
            FatType::Fat32 =>
                Some(
                    unsafe { self.bpb?.get_ext_bpb::<Extended32>()? }
                        .sectors_per_fat as usize
                ),

            _ => None
        }
    }

    pub fn get_total_sectors(&self) -> Option<usize> {
        if self.bpb?.low_sectors == 0 {
            Some(
                self.bpb?.high_sectors as usize
            )
        } else {
            Some(
                self.bpb?.low_sectors as usize
            )
        }
    }

    pub fn get_data_sector_count(&self) -> Option<usize> {
        let table_sectors =
            (self.bpb?.reserved_sectors as usize * self.get_fat_size()?)
                + self.get_root_dir_sector_count()?;
        let s_count = self.bpb?.reserved_sectors as usize + table_sectors;

        Some(
            self.get_total_sectors()? - s_count
        )
    }

    pub fn get_root_dir_sector_count(&self) -> Option<usize> {
        Some(
            (((self.bpb?.root_entries * 32) + (self.bpb?.bytes_per_sector - 1)) /
                (self.bpb?.bytes_per_sector)) as usize
        )
    }

    pub fn get_first_data_sector(&self) -> Option<usize> {
        let number_of_fat = self.bpb?.num_of_fats as usize;
        let fat_size = self.get_fat_size()?;
        let root_dir_sectors = self.get_root_dir_sector_count()?;

        Some(
            (number_of_fat * fat_size) + root_dir_sectors
        )
    }

    pub unsafe fn get_fat_table_sector(&self, sector_offset: usize) -> Option<[u32; 128]> {
        let sector = sector_offset + self.get_first_fat_sector()?;

        let mut sector_tmp = [0u32; 128];
        unsafe {
            self.disk.read_from_disk(&mut sector_tmp as *mut u32 as *mut u8,
                                     (sector as u16)..(sector as u16 + 1) );
        };

        Some(sector_tmp)
    }

    pub fn get_first_sector_of_cluster(&self, cluster: usize) -> Option<usize> {
        let rel_cluster = cluster - 2;

        Some(
            (self.bpb?.sectors_per_cluster as usize * rel_cluster) + self.get_first_data_sector()?
        )
    }

    pub fn read_root_dir(&self) -> Option<DirectoryEntry32>{
        let root_cluster_number = self.get_root_cluster_number()?;
        let root_sector = self.get_first_sector_of_cluster(root_cluster_number)?;

        let mut sector_tmp = [0u8; 512];
        unsafe {
            self.disk.read_from_disk(&mut sector_tmp as *mut u8,
                                     (root_sector as u16)..(root_sector as u16 + 1) );
        };

        bios_println!("test: {:?}",  sector_tmp);

        None
    }

    pub fn validate_fat(&self) -> FatType {
        // FIXME: make a better fat checker
        if let Some(bpb) = &self.bpb {

            // check if we understand this bpb (checking if this partition is fat)
            if !bpb.validate_fat() {
                return FatType::NotFat;
            }

            let fat_32_test = unsafe { bpb.get_ext_bpb::<Extended32>() };
            if let Some(fat32) = fat_32_test {
                if fat32.is_valid_sig() {
                    return FatType::Fat32;
                }
            }

            // TODO: Implement more fat filesystem types
            return FatType::NotImplemented;

        }

        FatType::NotFat
    }

    pub fn get_vol_label(&self) -> Option<CStringOwned> {
        let bpb_ref = &self.bpb?;

        let vol_label = match self.fat_type? {
            FatType::Fat32 => &unsafe { bpb_ref.get_ext_bpb::<Extended32>()? }.vol_label,

            _ => return None,
        };

        Some(unsafe { CStringOwned::from_ptr(vol_label as *const u8, vol_label.len()) } )
    }

    pub fn new_from_disk(disk_id: u8) -> Option<Self> {
        let mbr = unsafe { MasterBootRecord::read_from_disk(disk_id) };
        let partition =  mbr.get_partition_entry(
            mbr.get_bootable_partition()
                    .expect("No bootable partitions found!")
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

        match fat_type {
            FatType::Fat32 => Some(data),

            _ => None // TODO: Unsupported fat type
        }
    }

}



