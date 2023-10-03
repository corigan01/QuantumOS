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

use core::ptr;
use crate::error::{FsError, FsErrorKind};
use crate::filesystems::dosfs::structures::{Byte, ClusterID, DoubleWord, ExtendedBiosBlock, FatType, MAX_CLUSTERS_FOR_FAT12, MAX_CLUSTERS_FOR_FAT16, MAX_CLUSTERS_FOR_FAT32, Word};
use crate::filesystems::dosfs::structures::bpb16::ExtendedBPB16;
use crate::filesystems::dosfs::structures::bpb32::ExtendedBPB32;

pub union ExtendedBlock {
    fat16: ExtendedBPB16,
    fat32: ExtendedBPB32
}

pub enum WhereIsRoot {
    Cluster(ClusterID),
    OffsetBytes(usize)
}

#[repr(packed, C)]
pub struct BiosParameterBlock {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: Word,
    sectors_per_cluster: Byte,
    reserved_sector_count: Word,
    number_of_fats: Byte,
    root_entry_count: Word,
    total_sectors_16: Word,
    media: Byte,
    fat_sectors_16: Word,
    sectors_per_track: Word,
    number_of_heads: Word,
    hidden_sectors: DoubleWord,
    total_sectors_32: DoubleWord,
    extended: ExtendedBlock
}

impl BiosParameterBlock {
    pub fn verify_jmp_instruction(&self) -> bool {
        (self.jmp_boot[0] == 0xEB && self.jmp_boot[2] == 0x90) ||
            (self.jmp_boot[0] == 0xE9)
    }

    pub fn oem_name(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.oem_name) }
    }

    pub fn volume_label(&self) -> &str {
        unsafe {
            match self.fat_type() {
                FatType::Fat12 | FatType::Fat16 => {
                    self.extended.fat16.volume_label()
                }
                FatType::Fat32 => {
                    self.extended.fat32.volume_label()
                }
            }
        }
    }

    pub fn verify_sector_count_correctness(&self) -> bool {
        (self.total_sectors_16 == 0 && self.total_sectors_32 > 0) &&
            (self.total_sectors_32 == 0 && self.total_sectors_16 > 0)
    }

    pub fn total_sectors(&self) -> usize {
        // `total_sectors_16` will be zero when `total_sectors_32` is populated, and vis versa
        (self.total_sectors_16 as usize) | (self.total_sectors_32 as usize)
    }

    pub fn bytes_per_sector(&self) -> usize {
        self.bytes_per_sector as usize
    }

    pub fn total_bytes(&self) -> usize {
        self.total_sectors() * self.bytes_per_sector()
    }

    pub fn root_entries(&self) -> usize {
        self.root_entry_count as usize
    }

    pub fn root_directory_bytes(&self) -> usize {
        self.root_entries() * 32
    }

    pub fn bytes_to_sectors(&self, bytes: usize) -> usize {
        (bytes + (self.bytes_per_sector() - 1)) / self.bytes_per_sector()
    }

    pub fn root_directory_sectors(&self) -> usize {
        self.bytes_to_sectors(self.root_directory_bytes())
    }

    pub fn sectors_per_cluster(&self) -> usize {
        self.sectors_per_cluster as usize
    }

    pub fn bytes_per_cluster(&self) -> usize {
        self.bytes_per_sector() * self.sectors_per_cluster()
    }

    pub fn reserved_sectors(&self) -> usize {
        self.reserved_sector_count as usize
    }

    pub fn number_of_fats(&self) -> usize {
        self.number_of_fats as usize
    }

    pub fn fat_size_sectors(&self) -> usize {
        if self.fat_sectors_16 != 0 {
            self.fat_sectors_16 as usize
        } else {
            unsafe { self.extended.fat32.fat_sectors_32 as usize }
        }
    }

    pub fn fat_size_bytes(&self) -> usize {
        self.fat_size_sectors() * self.bytes_per_sector()
    }

    pub fn fat_begin_sector(&self) -> usize {
        self.reserved_sectors()
    }

    pub fn fat_begin_bytes(&self) -> usize {
        self.fat_begin_sector() * self.bytes_per_sector()
    }

    pub fn data_sectors(&self) -> usize {
        let total_fat_sectors = self.number_of_fats() * self.fat_size_sectors();
        let sectors_used_by_dosfs = self.reserved_sectors() + total_fat_sectors + self.root_directory_sectors();

        self.total_sectors() - sectors_used_by_dosfs
    }

    pub fn count_of_clusters(&self) -> usize {
        self.data_sectors() / self.sectors_per_cluster()
    }

    pub fn fat_type(&self) -> FatType {
        match self.count_of_clusters() {
            ..MAX_CLUSTERS_FOR_FAT12 => FatType::Fat12,
            ..MAX_CLUSTERS_FOR_FAT16 => FatType::Fat16,
            ..MAX_CLUSTERS_FOR_FAT32 => FatType::Fat32,

            _ => unreachable!("Cannot have more then Fat32 clusters in dosfs")
        }
    }

    fn root_offset_bytes_for_fat_12_16(&self) -> usize {
        (self.reserved_sectors() + self.fat_size_sectors()) * self.bytes_per_sector()
    }

    pub fn where_is_root(&self) -> WhereIsRoot {
        match self.fat_type() {
            FatType::Fat12 | FatType::Fat16 => WhereIsRoot::OffsetBytes(self.root_offset_bytes_for_fat_12_16()),
            FatType::Fat32 => unsafe {
                WhereIsRoot::Cluster(self.extended.fat32.root_cluster_number as usize)
            }
        }
    }
}

impl TryFrom<&[u8]> for BiosParameterBlock {
    type Error = FsError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 512 {
            return Err(FsError::try_from_array_error::<Self>(value));
        }

        let raw_bpb = unsafe { ptr::read_unaligned(value.as_ptr() as *const Self) };

        if !raw_bpb.verify_jmp_instruction() || !raw_bpb.verify_sector_count_correctness() {
            return Err(FsError::new(FsErrorKind::InvalidData,
                                    "Attempted BiosParameterBlock does not contain valid data. Not dosfs!"));
        }

        match raw_bpb.fat_type() {
            FatType::Fat12 | FatType::Fat16 => unsafe {
                if !raw_bpb.extended.fat16.verify() {
                    return Err(FsError::new(FsErrorKind::InvalidData,
                                            "Attempted Extended Fat12/Fat16 does not contain valid data. Not dosfs!"));
                }
            }
            FatType::Fat32 => unsafe {
                if !raw_bpb.extended.fat32.verify() {
                    return Err(FsError::new(FsErrorKind::InvalidData,
                                            "Attempted Extended Fat32 does not contain valid data. Not dosfs!"));
                }
            }
        }

        Ok(raw_bpb)
    }
}