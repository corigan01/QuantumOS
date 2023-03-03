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
use crate::mbr::{MasterBootRecord, PartitionEntry};

pub enum FatType {
    Fat16,
    Fat32,
    ExFat,
    VFat,

    NotFat
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct BiosParametersBlock {
    jmp_bytes: [u8; 3],
    oem_id: u64,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    num_of_fats: u8,
    root_entries: u16,
    low_sectors: u16,
    media_descriptor_type: u8,
    sectors_per_fat: u16,
    sectors_per_track: u16,
    heads_on_media: u16,
    hidden_sectors: u32,
    high_sectors: u32
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

    pub fn validate_fat(&self) -> FatType {

        // FIXME: make a better fat checker
        if let Some(bpb) = &self.bpb {

            // check if we understand this bpb (checking if this partition is fat)
            if  bpb.jmp_bytes[0] != 0xeb        ||
                bpb.bytes_per_sector != 512     ||
                bpb.oem_id == 0x00              ||
                bpb.media_descriptor_type == 0  ||
                !( bpb.low_sectors == 0 && bpb.high_sectors > 0 || bpb.low_sectors > 0)
            {
                return FatType::NotFat;
            }


        }

        FatType::NotFat
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

        bios_println!("\n{:#x?}", &data.bpb);

        data.validate_fat();

        None
    }

}



