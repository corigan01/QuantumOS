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

use crate::error::BootloaderError;
use crate::filesystem::partition::{PartitionEntry, Partitions, PartitionType};

#[derive(Copy, Clone, Debug)]
enum MBRPartitionTypes {
    Zero,
    LinuxNative,
    OsdevGang,
    Unknown(u8),
}

impl MBRPartitionTypes {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x00 => MBRPartitionTypes::Zero,
            0x83 => MBRPartitionTypes::LinuxNative,
            0x7f => MBRPartitionTypes::OsdevGang,
            
            _ => MBRPartitionTypes::Unknown(value),
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct MBRPartitionEntry {
    drive_attributes: u8,
    chs_partition_start: u16,
    chs_partition_start_high: u8,
    partition_type: u8,
    chs_partition_end: u16,
    chs_partition_end_high: u8,
    lba_start: u32,
    total_sectors: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MasterBootRecord {
    boot_strap_sector: [u8; 440],
    disk_id: u32,
    optional: u16,
    partitions: [MBRPartitionEntry; 4],
}

impl MBRPartitionEntry {
    fn new() -> Self {
        Self {
            drive_attributes: 0,
            chs_partition_start: 0,
            chs_partition_start_high: 0,
            partition_type: 0,
            chs_partition_end: 0,
            chs_partition_end_high: 0,
            lba_start: 0,
            total_sectors: 0,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.partition_type > 0
    }

    pub fn is_bootable(&self) -> bool {
        self.drive_attributes > 0
    }

    pub fn get_partition_type(&self) -> MBRPartitionTypes {
        MBRPartitionTypes::from_u8(self.partition_type)
    }

    pub fn get_sector_start(&self) -> usize {
        self.lba_start as usize
    }
    
    pub fn get_sector_count(&self) -> usize {
        self.total_sectors as usize
    }

    pub fn get_sector_end(&self) -> usize {
        self.get_sector_start() + self.get_sector_count()
    }
    
}

impl MasterBootRecord {
    pub fn new(boot_sector: [u8; 512]) -> Self {
        // TODO: ensure that this consumes the boot_sector var properly 
        unsafe {
            *(boot_sector.as_mut_ptr() as *mut Self)
        }
    }
    
    pub fn is_valid(&self) -> bool {
        self.partitions[0].is_valid() ||
            self.partitions[1].is_valid() ||
            self.partitions[2].is_valid() ||
            self.partitions[3].is_valid()
    }

    pub fn get_partition_entry(&self, entry: usize) -> Result<&MBRPartitionEntry, BootloaderError> {
        if entry >= self.partitions.len() {
            return Err(BootloaderError::OutOfBounds);
        }

        Ok(&self.partitions[entry])
    }

    pub fn total_valid_partitions(&self) -> usize {
        let mut valid_partitions = 0;
        for i in &self.partitions {
            if i.is_valid() {
                valid_partitions += 1;
            }
        }

        valid_partitions
    }

    pub fn get_bootable_partition(&self) -> Option<usize> {
        for i in 0..4 {
            let partition = &self.partitions[i];

            if partition.is_valid() && partition.is_bootable() {
                return Some(i);
            }
        }

        None
    }
    
    pub fn to_partitions(self) -> Option<Partitions> {
        if !self.is_valid() {
            return None;
        }
        
        let mut building_partition_entries = [PartitionEntry::new(); 4];
        
        for i in 0..building_partition_entries.len() {
            let partition_entry_ref = &mut building_partition_entries[i];
            
            if let Some(our_partition_ref) = self.partitions.get_mut(i) {
                let start_sector = our_partition_ref.get_sector_start();
                let sector_end = our_partition_ref.get_sector_end();
                let sector_valid = our_partition_ref.is_valid();
                let sector_bootable = our_partition_ref.is_bootable();
                
                if !sector_valid {
                    partition_entry_ref.kind = PartitionType::Unknown;
                    continue;
                }

                partition_entry_ref.start_sector = Some(start_sector);
                partition_entry_ref.end_sector = Some(sector_end);
                partition_entry_ref.kind = 
                    if sector_bootable { PartitionType::Bootable } else { PartitionType::Normal };
                
            } else {
                partition_entry_ref.kind = PartitionType::None;
            }
        }
        
        Some(Partitions {
            partitions_array: building_partition_entries
        })
    }
}