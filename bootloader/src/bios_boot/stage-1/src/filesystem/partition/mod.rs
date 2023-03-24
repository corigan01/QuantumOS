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
use crate::filesystem::partition::mbr::MasterBootRecord;
use crate::filesystem::DiskMedia;

pub mod mbr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PartitionType {
    // Valid Checked Types
    Bootable,
    Normal,
    Unknown,
    None,

    // Unchecked Types
    NotChecked,
}

impl PartitionType {
    pub fn new() -> Self {
        Self::NotChecked
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PartitionEntry {
    start_sector: Option<usize>,
    end_sector: Option<usize>,
    kind: PartitionType,
}

impl PartitionEntry {
    pub fn new() -> Self {
        Self {
            start_sector: None,
            end_sector: None,
            kind: PartitionType::new(),
        }
    }

    pub fn get_partition_size(&self) -> Option<usize> {
        Some(self.end_sector? - self.start_sector?)
    }

    pub fn get_start_sector(&self) -> Option<usize> {
        self.start_sector
    }

    pub fn get_end_sector(&self) -> Option<usize> {
        self.end_sector
    }

    pub fn is_bootable(&self) -> bool {
        self.kind == PartitionType::Bootable
    }
}

pub struct Partitions {
    // FIXME: Should not be a constant size
    partitions_array: [PartitionEntry; 4],
}

impl Partitions {
    pub fn check_all<DiskType: DiskMedia>(disk: DiskType) -> Result<Self, BootloaderError> {
        if let Ok(mbr) = Self::check_mbr(disk) {
            return Ok(mbr);
        }

        Err(BootloaderError::NoValid)
    }

    pub fn check_mbr<DiskType: DiskMedia>(disk: DiskType) -> Result<Self, BootloaderError> {
        let boot_sector_data = disk.read(0)?;

        let test_mbr = MasterBootRecord::new(boot_sector_data);
        if let Some(partitions) = test_mbr.to_partitions() {
            return Ok(partitions);
        }

        Err(BootloaderError::NotSupported)
    }

    pub fn get_partitions_ref(&self) -> &[PartitionEntry] {
        self.partitions_array.as_ref()
    }

    pub fn get_number_of_partitions(&self) -> usize {
        self.partitions_array.len()
    }

    pub fn get_partition_entry(&self, index: usize) -> Option<&PartitionEntry> {
        self.partitions_array.get(index)
    }
}
