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

use core::mem::size_of;
use crate::bios_println;
use crate::cstring::CStringOwned;
use crate::error::BootloaderError;
use crate::filesystem::fat::fat16::ExtendedBPB16;
use crate::filesystem::fat::fat_file::{FatFile, FatFileType};
use crate::filesystem::fat::FatType;
use crate::filesystem::fat::FatValid;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct BiosBlockLow {
    pub jmp_bytes: [u8; 3],
    pub oem_id: u64,
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_of_fats: u8,
    pub root_entries: u16,
    pub low_sectors: u16,
    pub media_descriptor_type: u8,
    pub sectors_per_fat: u16,
    sectors_per_track: u16,
    heads_on_media: u16,
    pub hidden_sectors: u32,
    pub high_sectors: u32,

    ex_block: [u8; 54]
}

#[derive(Debug, Clone, Copy)]
pub struct BiosBlock {
    pub extended_type: FatType,
    pub bios_block: BiosBlockLow,
}


#[derive(Debug)]
pub enum FatTableEntryType {
    Valid(usize),
    Reserved,
    SectorError,
    EndOfFile
}


impl BiosBlock {
    pub fn convert_usize_to_fat_table_entry(&self, fat_table: usize) -> Result<FatTableEntryType, BootloaderError> {
        Ok(match self.extended_type {
            FatType::Fat16 => {
                match fat_table {
                    0xfff8 | 0xffff => FatTableEntryType::EndOfFile,
                    0xfff7 => FatTableEntryType::SectorError,
                    0 | 1  => FatTableEntryType::Reserved,

                    _ => FatTableEntryType::Valid(fat_table)
                }
            }


            _ => return Err(BootloaderError::NoValid)
        })
    }

    pub unsafe fn convert_extended_type_unchecked<ExtendedType: FatValid + Copy>(&self) -> ExtendedType {
        let exb = &self.bios_block.ex_block;
        let ptr = exb.as_ptr() as *const ExtendedType;

        *ptr
    }

    pub fn convert_extended_type<ExtendedType: FatValid + Copy>(&self) -> Result<ExtendedType, BootloaderError>
        where ExtendedType: FatValid + Copy {

        let unchecked_value = unsafe { self.convert_extended_type_unchecked::<ExtendedType>() };

        if ExtendedType::is_valid(self) {
            return Ok(unchecked_value);
        }

        Err(BootloaderError::NoValid)
    }

    unsafe fn get_raw_file_allocation_table_entry<EntrySize: Copy>(offset: usize, data: &[u8]) -> Result<EntrySize, BootloaderError> {
        let entry_bytes = size_of::<EntrySize>();
        let offset_bytes = entry_bytes * offset;

        if offset_bytes > data.len() {
            return Err(BootloaderError::OutOfBounds);
        }

        let raw_ptr = data.as_ptr().add(offset_bytes) as *const EntrySize;

        Ok(*raw_ptr)
    }

    pub fn get_file_allocation_table_entry_size_bytes(&self) -> Result<usize, BootloaderError> {
        Ok(
            match self.extended_type {
                FatType::Fat16 => {
                    size_of::<u16>()
                }

                _ => return Err(BootloaderError::NoValid)
            }
        )
    }

    pub fn get_file_allocation_table_entry(&self, offset: usize, fat_data: &[u8]) -> Result<usize, BootloaderError> {
        Ok(
            match self.extended_type {
                FatType::Fat12 | FatType::Fat16 => {
                    (unsafe { Self::get_raw_file_allocation_table_entry::<u16>(offset, fat_data)? }) as usize
                }

                _ => return Err(BootloaderError::NoValid)
            }
        )
    }


    pub fn root_sector_count(&self) -> Result<usize, BootloaderError> {
        Ok(
            match self.extended_type {
                FatType::Fat12 | FatType::Fat16 => {
                    let root_entries = self.bios_block.root_entries as usize;
                    let bytes_per_sector = self.bios_block.bytes_per_sector as usize;

                    let root_bytes = root_entries * 32;

                    (root_bytes + (bytes_per_sector - 1)) / bytes_per_sector
                }


                _ => return Err(BootloaderError::NoValid)
            }
        )
    }

    pub fn file_allocation_table_begin(&self) -> Result<usize, BootloaderError> {
        Ok(match self.extended_type {
            FatType::Fat12 | FatType::Fat16 => {
                self.bios_block.reserved_sectors as usize
            }

            _ => return Err(BootloaderError::NoValid)
        })
    }

    pub fn file_allocation_table_count(&self) -> Result<usize, BootloaderError> {
        Ok(
            match self.extended_type {
                FatType::Fat12 | FatType::Fat16 => {
                    self.bios_block.num_of_fats as usize
                }

                _ => return Err(BootloaderError::NoValid)
            }
        )
    }

    pub fn file_allocation_table_size(&self) -> Result<usize, BootloaderError> {
        Ok(
            match self.extended_type {
                FatType::Fat12 | FatType::Fat16 => {
                    self.bios_block.sectors_per_fat as usize
                }

                _ => return Err(BootloaderError::NoValid)
            }
        )
    }

    pub fn cluster_size(&self) -> Result<usize, BootloaderError> {
        Ok(match self.extended_type {
            FatType::Fat12 | FatType::Fat16 => {
                self.bios_block.sectors_per_cluster as usize
            }

            _ => return Err(BootloaderError::NoValid)
        })
    }

    pub fn reserved_sectors(&self) -> usize {
        self.bios_block.reserved_sectors as usize
    }

    pub fn data_cluster_begin(&self) -> Result<usize, BootloaderError> {
        let fat_table_size = self.file_allocation_table_size()?;
        let fat_table_count = self.file_allocation_table_count()?;
        let total_table_size = fat_table_count * fat_table_size;

        let root_sectors = self.root_sector_count()?;
        let reserved_sectors =  self.reserved_sectors();

        Ok(
            total_table_size
            + reserved_sectors
            + root_sectors
        )
    }

    pub fn root_cluster_begin(&self) -> Result<usize, BootloaderError> {
        let data_cluster = self.data_cluster_begin()?;
        let root_cluster_size = self.root_sector_count()?;

        Ok(data_cluster - root_cluster_size)
    }

    pub fn get_root_file_entry(&self) -> Result<FatFile, BootloaderError> {
        let root_cluster = self.root_cluster_begin()?;
        let root_size = self.root_sector_count()?;


        Ok(
            FatFile {
                filename: CStringOwned::from_static_bytes("/".as_bytes()),
                start_cluster: root_cluster,
                filesize_bytes: root_size,
                filetype: FatFileType::Root,
            }
        )
    }
    
    pub fn get_fat_type(&self) -> FatType {
        self.extended_type
    }

    pub fn new(mut boot_sector: [u8; 512]) -> Self {
        let bpb_low = unsafe {
            *(boot_sector.as_mut_ptr() as *mut BiosBlockLow)
        };

        let mut bp = Self {
            extended_type: FatType::Unknown,
            bios_block: bpb_low
        };

        if ExtendedBPB16::is_valid(&bp) {
            bp.extended_type = FatType::Fat16;
        }

        bp
    }


}

