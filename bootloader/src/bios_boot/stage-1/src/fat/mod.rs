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
use crate::bios_println;
use crate::cstring::{CStringRef, CStringOwned};
use crate::fat::bpb::BiosParametersBlock;
use crate::fat::fat_32::Extended32;
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
    pub bpb: Option<BiosParametersBlock>,
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
        let ext_bpb = unsafe {
            bpb_ref.get_ext_bpb::<Extended32>()?
        };

        let vol_label = &ext_bpb.vol_label;

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
            FatType::NotFat => None,

            _ => None // TODO: Unsupported fat type
        }
    }

}



