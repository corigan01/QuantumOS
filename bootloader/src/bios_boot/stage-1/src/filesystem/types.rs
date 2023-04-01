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

use crate::bios_println;
use crate::error::BootloaderError;
use crate::filesystem::fat::Fatfs;
use crate::filesystem::{DiskMedia, PartitionEntry, ValidFilesystem};

#[derive(Clone, Copy)]
pub enum FileSystemTypes {
    Fat(PartitionEntry),

    NotSupported,
    Unchecked,
}

impl FileSystemTypes {
    pub fn new() -> Self {
        Self::Unchecked
    }

    pub fn get_filesize_bytes<DiskType: DiskMedia>(
        &self,
        disk: &DiskType,
        filename: &str,
    ) -> Result<usize, BootloaderError> {
        let bytes = match self {
            FileSystemTypes::Fat(partition) => {
                <Fatfs<DiskType> as ValidFilesystem<DiskType>>::size_of_file(
                    &disk, &partition, filename,
                )
            }

            _ => Err(BootloaderError::NoValid),
        };

        bytes
    }

    pub fn get_partition_entry(&self) -> Option<&PartitionEntry> {
        match self {
            FileSystemTypes::Fat(entry) => Some(entry),

            _ => None,
        }
    }

    pub fn does_contain_file<DiskType: DiskMedia>(
        &self,
        disk: &DiskType,
        filename: &str,
    ) -> Result<bool, BootloaderError> {
        if let Some(partition_entry) = self.get_partition_entry() {
            // FIXME: Make this type independent, so that we can call 'does_contain_file' on
            // any filesystem that supports this trait
            let fat_test = <Fatfs<DiskType> as ValidFilesystem<DiskType>>::does_contain_file(
                disk,
                partition_entry,
                filename,
            )?;

            return Ok(fat_test);
        }

        Err(BootloaderError::NoValid)
    }

    pub unsafe fn load_file_to_ptr<DiskType: DiskMedia>(
        &self,
        disk: &DiskType,
        filename: &str,
        ptr: *mut u8,
    ) -> Result<(), BootloaderError> {
        match self {
            FileSystemTypes::Fat(partition) => {
                <Fatfs<DiskType> as ValidFilesystem<DiskType>>::load_file_to_ptr(
                    disk, partition, filename, ptr,
                )?;
            }

            _ => return Err(BootloaderError::NoValid),
        }

        Ok(())
    }

    pub fn get_volume_name<DiskType: DiskMedia>(
        &self,
        disk: &DiskType,
    ) -> Result<[u8; 11], BootloaderError> {
        match self {
            FileSystemTypes::Fat(partition) => {
                <Fatfs<DiskType> as ValidFilesystem<DiskType>>::get_volume_name(disk, partition)
            }

            _ => return Err(BootloaderError::NoValid),
        }
    }
}
