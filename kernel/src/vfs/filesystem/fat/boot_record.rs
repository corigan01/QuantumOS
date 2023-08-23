/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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
use core::ptr;
use qk_alloc::boxed::Box;
use qk_alloc::string::String;
use qk_alloc::vec::Vec;
use crate::vfs::filesystem::fat::{EntryType, FatEntry};
use crate::vfs::filesystem::FilesystemError;
use crate::vfs::io::{IOError, IOResult, SeekFrom};
use crate::vfs::VFSPartition;

#[repr(C, packed)]
#[derive(Debug)]
struct RawBiosParameterBlock {
    reserved: [u8; 3],
    oem_id: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fat_tables: u8,
    root_dir_entries: u16,
    logical_sectors: u16,
    media_descriptor_type: u8,
    sectors_per_fat: u16,
    sectors_per_track: u16,
    number_of_heads: u16,
    hidden_sectors: u32,
    large_sector_count: u32
}

impl RawBiosParameterBlock {
    const RESERVED_INSTRUCTIONS: [u8; 3] = [0xEB, 0x3C, 0x90];

    pub fn is_valid(&self) -> bool {
        self.fat_tables != 0 && self.reserved == Self::RESERVED_INSTRUCTIONS
    }
}

#[repr(C, packed)]
#[derive(Debug)]
struct RawExtendedBootRecord16 {
    drive_number: u8,
    reserved: u8,
    signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    system_identifier_string: u64,
}

impl RawExtendedBootRecord16 {
    const VALID_SIG_1: u8 = 0x28;
    const VALID_SIG_2: u8 = 0x29;

    pub fn is_valid(&self) -> bool {
        self.signature == Self::VALID_SIG_1 || self.signature == Self::VALID_SIG_2
    }
}

impl TryFrom<&[u8]> for RawExtendedBootRecord16 {
    type Error = Box<dyn IOError>;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < size_of::<Self>() {
            return Err(Box::new(FilesystemError::Invalid));
        }

        let our_copy = unsafe { ptr::read(value.as_ptr() as *const Self) };

        if !our_copy.is_valid() {
            return Err(Box::new(FilesystemError::Invalid));
        }

        Ok(our_copy)
    }
}

impl FatProvider for RawExtendedBootRecord16 {
    fn fat_type(&self) -> FatType {
        FatType::Fat16
    }

    fn volume_label(&self) -> String {
        String::from(unsafe {
            core::str::from_utf8_unchecked(&self.volume_label)
        })
    }
}

pub enum FatType {
    Unknown,
    Fat16,
}

pub trait FatProvider {
    fn fat_type(&self) -> FatType;
    fn volume_label(&self) -> String;
}

pub struct BiosParameterBlock {
    bpb: RawBiosParameterBlock,
    extended: Box<dyn FatProvider>
}

impl BiosParameterBlock {
    pub fn populate_from_medium(medium: &mut Box<dyn VFSPartition>) -> IOResult<Self> {
        medium.seek(SeekFrom::Start(0))?;

        let mut first_sector = Vec::from([0u8; 512]);
        medium.read_exact(&mut first_sector)?;

        let bpb = unsafe { ptr::read(first_sector.as_ptr() as *const RawBiosParameterBlock) };

        if !bpb.is_valid() {
            return Err(Box::new(FilesystemError::Invalid));
        }

        // More filesystems will be supported here
        let try_bpb16 = Box::new(
            RawExtendedBootRecord16::try_from(&first_sector.as_slice()[36..512])?
        );

        Ok(Self {
            bpb,
            extended: try_bpb16
        })
    }

    pub fn reserved_sectors(&self) -> u64 {
        self.bpb.reserved_sectors as u64
    }

    pub fn file_allocation_table_size(&self) -> u64 {
        self.bpb.sectors_per_fat as u64
    }

    pub fn number_of_file_allocation_tables(&self) -> u64 {
        self.bpb.fat_tables as u64
    }

    pub fn file_allocation_table_offset(&self) -> u64 {
        self.reserved_sectors()
    }

    pub fn cluster_size(&self) -> u64 {
        self.bpb.sectors_per_cluster as u64
    }

    pub fn root_sectors(&self) -> u64 {
        let root_entries = self.bpb.root_dir_entries as u64;
        let bytes_per_sector = self.bpb.bytes_per_sector as u64;

        let root_bytes = root_entries * 32;

        (root_bytes + (bytes_per_sector - 1)) / bytes_per_sector
    }

    pub fn data_cluster_offset(&self) -> u64 {
        let fats = self.number_of_file_allocation_tables();
        let fat_size = self.file_allocation_table_size();

        let table_sectors = fats * fat_size;
        let root_sectors = self.root_sectors();
        let reserved_sectors = self.reserved_sectors();

        table_sectors + root_sectors + reserved_sectors
    }

    pub fn root_cluster_begin(&self) -> u64 {
        let data_cluster_begin = self.data_cluster_offset();
        let root_sectors = self.root_sectors();

        data_cluster_begin - root_sectors
    }

    pub fn get_root_entry(&self) -> FatEntry {
        let root_cluster = self.root_cluster_begin();
        let root_sectors = self.root_sectors();

        FatEntry {
            path: String::from("/"),
            start_cluster: root_cluster,
            sector_count: root_sectors,
            kind: EntryType::RootDir,
        }
    }

    pub fn fat_kind(&self) -> FatType {
        self.extended.fat_type()
    }
}
