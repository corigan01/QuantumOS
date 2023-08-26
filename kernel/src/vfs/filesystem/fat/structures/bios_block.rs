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

use core::ptr;
use qk_alloc::boxed::Box;
use qk_alloc::string::String;
use qk_alloc::vec::Vec;

use crate::vfs::filesystem::fat::structures::{EntryType, FatProvider, FatType, FileEntry};
use crate::vfs::filesystem::fat::structures::raw::{RawBiosParameterBlock, RawExtendedBootRecord16};
use crate::vfs::filesystem::FilesystemError;
use crate::vfs::io::{IOResult, SeekFrom};
use crate::vfs::{VFSPartition, VFSPartitionID};

pub struct BiosParameterBlock {
    bpb: RawBiosParameterBlock,
    extended: Box<dyn FatProvider>
}

impl BiosParameterBlock {
    pub fn populate_from_media(medium: &mut Box<dyn VFSPartition>) -> IOResult<Self> {
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

    pub fn get_root_entry(&self) -> FileEntry {
        let root_cluster = self.root_cluster_begin();
        let root_sectors = self.root_sectors();

        FileEntry {
            path: String::from("/"),
            start_cluster: root_cluster,
            sector_count: root_sectors,
            kind: EntryType::RootDir,
        }
    }

    pub fn total_sectors(&self) -> usize {
        if self.bpb.logical_sectors == 0 {
            self.bpb.large_sector_count as usize
        } else {
            self.bpb.logical_sectors as usize
        }
    }

    pub fn fat_kind(&self) -> FatType {
        self.extended.fat_type()
    }
}
