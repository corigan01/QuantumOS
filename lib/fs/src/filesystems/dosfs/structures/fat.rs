/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
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

use crate::abstract_buffer::AbstractBuffer;
use crate::error::{FsError, FsErrorKind};
use crate::filesystems::dosfs::structures::{FatType, MAX_CLUSTERS_FOR_FAT12, MAX_CLUSTERS_FOR_FAT16, MAX_CLUSTERS_FOR_FAT32};
use crate::FsResult;
use crate::io::{ReadWriteSeek, SeekFrom};


pub enum FatEntry {
    Free,
    NextCluster(usize),
    Reserved,
    Defective,
    ReservedEndOfFile,
    EndOfFile,
}

impl FatEntry {
    const FAT_12_FREE_CLUSTER: usize = 0x000;
    const FAT_12_ALLOCATED_LOW: usize = 0x000;
    const FAT_12_ALLOCATED_HIGH: usize = MAX_CLUSTERS_FOR_FAT12;
    const FAT_12_RESERVED_LOW: usize = MAX_CLUSTERS_FOR_FAT12 + 1;
    const FAT_12_RESERVED_HIGH: usize = 0xFF6;
    const FAT_12_DEFECTIVE_CLUSTER: usize = 0xFF7;
    const FAT_12_RESERVED_EOF_LOW: usize = 0xFF8;
    const FAT_12_RESERVED_EOF_HIGH: usize = 0xFFE;
    const FAT_12_END_OF_FILE_CLUSTER: usize = 0xFFF;

    const FAT_16_FREE_CLUSTER: usize = 0x0000;
    const FAT_16_ALLOCATED_LOW: usize = 0x0000;
    const FAT_16_ALLOCATED_HIGH: usize = MAX_CLUSTERS_FOR_FAT16;
    const FAT_16_RESERVED_LOW: usize = MAX_CLUSTERS_FOR_FAT16 + 1;
    const FAT_16_RESERVED_HIGH: usize = 0xFFF6;
    const FAT_16_DEFECTIVE_CLUSTER: usize = 0xFFF7;
    const FAT_16_RESERVED_EOF_LOW: usize = 0xFFF8;
    const FAT_16_RESERVED_EOF_HIGH: usize = 0xFFFE;
    const FAT_16_END_OF_FILE_CLUSTER: usize = 0xFFFF;

    const FAT_32_FREE_CLUSTER: usize = 0x0000;
    const FAT_32_ALLOCATED_LOW: usize = 0x0000;
    const FAT_32_ALLOCATED_HIGH: usize = MAX_CLUSTERS_FOR_FAT32;
    const FAT_32_RESERVED_LOW: usize = MAX_CLUSTERS_FOR_FAT32 + 1;
    const FAT_32_RESERVED_HIGH: usize = 0xFFFFFF6;
    const FAT_32_DEFECTIVE_CLUSTER: usize = 0xFFFFFF7;
    const FAT_32_RESERVED_EOF_LOW: usize = 0xFFFFFF8;
    const FAT_32_RESERVED_EOF_HIGH: usize = 0xFFFFFFE;
    const FAT_32_END_OF_FILE_CLUSTER: usize = 0xFFFFFFFF;

    fn from_fat12(value: usize) -> Self {
        match value {
            Self::FAT_12_FREE_CLUSTER => Self::Free,
            Self::FAT_12_ALLOCATED_LOW..=Self::FAT_12_ALLOCATED_HIGH => Self::NextCluster(value),
            Self::FAT_12_RESERVED_LOW..=Self::FAT_12_RESERVED_HIGH => Self::Reserved,
            Self::FAT_12_DEFECTIVE_CLUSTER => Self::Defective,
            Self::FAT_12_RESERVED_EOF_LOW..=Self::FAT_12_RESERVED_EOF_HIGH => Self::ReservedEndOfFile,
            Self::FAT_12_END_OF_FILE_CLUSTER => Self::EndOfFile,
            _ => unreachable!("Value Out Of Range for Fat12")
        }
    }

    fn from_fat16(value: usize) -> Self {
        match value {
            Self::FAT_16_FREE_CLUSTER => Self::Free,
            Self::FAT_16_ALLOCATED_LOW..=Self::FAT_16_ALLOCATED_HIGH => Self::NextCluster(value),
            Self::FAT_16_RESERVED_LOW..=Self::FAT_16_RESERVED_HIGH => Self::Reserved,
            Self::FAT_16_DEFECTIVE_CLUSTER => Self::Defective,
            Self::FAT_16_RESERVED_EOF_LOW..=Self::FAT_16_RESERVED_EOF_HIGH => Self::ReservedEndOfFile,
            Self::FAT_16_END_OF_FILE_CLUSTER => Self::EndOfFile,
            _ => unreachable!("Value Out Of Range for Fat16")
        }
    }

    fn from_fat32(value: usize) -> Self {
        match value {
            Self::FAT_32_FREE_CLUSTER => Self::Free,
            Self::FAT_32_ALLOCATED_LOW..=Self::FAT_32_ALLOCATED_HIGH => Self::NextCluster(value),
            Self::FAT_32_RESERVED_LOW..=Self::FAT_32_RESERVED_HIGH => Self::Reserved,
            Self::FAT_32_DEFECTIVE_CLUSTER => Self::Defective,
            Self::FAT_32_RESERVED_EOF_LOW..=Self::FAT_32_RESERVED_EOF_HIGH => Self::ReservedEndOfFile,
            Self::FAT_32_END_OF_FILE_CLUSTER => Self::EndOfFile,
            _ => unreachable!("Value Out Of Range for Fat32")
        }
    }
}


pub struct FileAllocationTable {
    fat_type: FatType,
    fat_begin: u64,
    fat_size: u64
}

impl FileAllocationTable {
    pub fn new(fat_type: FatType, fat_begin: u64, fat_size: u64) -> Self {
        Self {
            fat_type,
            fat_begin,
            fat_size
        }
    }

    fn read_fat12_entry<Reader>(&self, index: usize, reader: &mut Reader) -> FsResult<FatEntry>
        where Reader: ReadWriteSeek {
        todo!("FAT12 fat entry")
    }

    fn read_fat16_entry<Reader>(&self, index: usize, reader: &mut Reader) -> FsResult<FatEntry>
        where Reader: ReadWriteSeek {
        if index >= MAX_CLUSTERS_FOR_FAT16 {
            return Err(FsError::new(FsErrorKind::InvalidInput,
                                    "Cannot address more then 65525 clusters in fat16"));
        }

        let byte_offset = index * 2;
        let mut byte_slice = [0; 2];
        reader.seek(SeekFrom::Start(byte_offset as u64))?;
        reader.read(&mut byte_slice)?;

        let large_value = u16::from_le_bytes(byte_slice) as usize;

        Ok(FatEntry::from_fat16(large_value))
    }

    fn read_fat32_entry<Reader>(&self, index: usize, reader: &mut Reader) -> FsResult<FatEntry>
        where Reader: ReadWriteSeek {
        if index >= MAX_CLUSTERS_FOR_FAT32 {
            return Err(FsError::new(FsErrorKind::InvalidInput,
                                    "Cannot address more then 268435447 clusters in fat32"));
        }

        let byte_offset = index * 4;

        let mut byte_slice = [0; 4];
        reader.seek(SeekFrom::Start(byte_offset as u64))?;
        reader.read(&mut byte_slice)?;

        let large_value = u32::from_le_bytes(byte_slice) as usize;

        Ok(FatEntry::from_fat32(large_value))
    }

    pub fn read_entry(&self, index: usize, reader: &mut AbstractBuffer) -> FsResult<FatEntry> {
        let new_range = self.fat_begin..=(self.fat_size + self.fat_begin);

        reader.temporary_shrink(new_range, |reader| {
            match self.fat_type {
                FatType::Fat12 => self.read_fat12_entry(index, reader),
                FatType::Fat16 => self.read_fat16_entry(index, reader),
                FatType::Fat32 => self.read_fat32_entry(index, reader),
            }
        })
    }
}
