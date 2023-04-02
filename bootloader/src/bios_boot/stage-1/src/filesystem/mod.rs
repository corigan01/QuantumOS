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

pub mod fat;
pub mod partition;
pub mod types;

use crate::error::BootloaderError;
use crate::{bios_print, bios_println};

use crate::filesystem::fat::Fatfs;
use crate::filesystem::partition::{PartitionEntry, Partitions};
use core::marker::PhantomData;
use types::FileSystemTypes;

pub struct UnQuarried;
pub struct Quarried;
pub struct MountedRoot;

pub trait DiskMedia {
    // FIXME: sector size should not be defined to always 512
    fn read(&self, sector: usize) -> Result<[u8; 512], BootloaderError>;

    unsafe fn read_ptr(&self, sector: usize, ptr: *mut u8) -> Result<(), BootloaderError>;
}

pub trait ValidFilesystem<DiskType: DiskMedia> {
    fn is_valid(disk: &DiskType, partition: &PartitionEntry) -> bool;

    // FIXME: Should not have a hard limit of 32 characters
    fn get_volume_name(
        disk: &DiskType,
        partition: &PartitionEntry,
    ) -> Result<[u8; 11], BootloaderError>;
    fn does_contain_file(
        disk: &DiskType,
        partition: &PartitionEntry,
        filename: &str,
    ) -> Result<bool, BootloaderError>;
    fn size_of_file(
        disk: &DiskType,
        partition: &PartitionEntry,
        filename: &str,
    ) -> Result<usize, BootloaderError>;
    unsafe fn load_file_to_ptr(
        disk: &DiskType,
        partition: &PartitionEntry,
        filename: &str,
        ptr: *mut u8,
    ) -> Result<(), BootloaderError>;
}

pub struct FileSystem<DiskType, State = UnQuarried>
where
    DiskType: Sized + DiskMedia,
{
    // FIXME: Should not be a hard defined limit on how many filesystems can be attached
    current_filesystems: [FileSystemTypes; 4],
    root: FileSystemTypes,
    attached_disk: DiskType,
    logging_enable: bool,
    state: PhantomData<State>,
}

impl<DiskType: DiskMedia> FileSystem<DiskType> {
    pub fn new(disk: DiskType) -> FileSystem<DiskType> {
        FileSystem {
            current_filesystems: [FileSystemTypes::new(); 4],
            root: FileSystemTypes::Unchecked,
            attached_disk: disk,
            logging_enable: false,
            state: Default::default(),
        }
    }
}

impl<D: DiskMedia, T> FileSystem<D, T> {
    pub fn toggle_logging(mut self) -> Self {
        self.logging_enable = !self.logging_enable;

        self
    }
}

impl<DiskType: DiskMedia + Clone> FileSystem<DiskType, UnQuarried> {
    pub fn quarry_disk(mut self) -> Result<FileSystem<DiskType, Quarried>, BootloaderError> {
        if self.logging_enable {
            bios_println!("Disk Quarry: ");
        }

        let partitions = Partitions::check_all(self.attached_disk.clone())?;
        let entries_ref = partitions.get_partitions_ref();
        let mut did_find_fs = 0;

        for (i, partition) in entries_ref.iter().enumerate() {
            // FIXME: Consider adding a macro that adds these automatically instead of having
            // to add new filesystem drivers here each time
            if Fatfs::<DiskType>::is_valid(&self.attached_disk, partition) {
                self.current_filesystems[i] = FileSystemTypes::Fat(partition.clone());
                did_find_fs += 1;
            }

            if self.logging_enable && partition.get_start_sector().unwrap_or(0) != 0 {
                bios_println!(
                    "    [{}]: '{:5}' {:7}MiB {:10} {:10}",
                    i,
                    core::str::from_utf8(
                        &self.current_filesystems[i]
                            .get_volume_name(&self.attached_disk)
                            .unwrap_or([b' '; 11])
                    )
                    .unwrap(),
                    partition.get_partition_size().unwrap_or(0) / 2048,
                    if partition.is_bootable() {
                        "bootable"
                    } else {
                        "normal"
                    },
                    match self.current_filesystems[i] {
                        FileSystemTypes::Fat(_) => "Fat",
                        _ => "Unknown Filesystem!",
                    }
                );
            }
        }

        if did_find_fs != 0 {
            if self.logging_enable {
                bios_println!("\nFound {} valid/supported filesystem(s)!\n", did_find_fs);
            }

            Ok(FileSystem::<DiskType, Quarried> {
                current_filesystems: self.current_filesystems,
                root: FileSystemTypes::Unchecked,
                attached_disk: self.attached_disk.clone(),
                logging_enable: self.logging_enable,
                state: PhantomData::<Quarried>,
            })
        } else {
            Err(BootloaderError::NoValid)
        }
    }
}

impl<DiskType: DiskMedia + Clone> FileSystem<DiskType, Quarried> {
    pub fn mount_root_if_contains(
        &self,
        filename: &str,
    ) -> Result<FileSystem<DiskType, MountedRoot>, BootloaderError> {
        if self.logging_enable {
            bios_println!("Looking for '{}' on all disks: ", filename);
        }
        for (i, filesystems) in self.current_filesystems.iter().enumerate() {
            let contains_file = filesystems.does_contain_file(&self.attached_disk, filename)?;

            if self.logging_enable {
                bios_println!(
                    "    [{}] '{}' {}",
                    i,
                    core::str::from_utf8(
                        &self.current_filesystems[i]
                            .get_volume_name(&self.attached_disk)
                            .unwrap_or([b' '; 11])
                    )
                    .unwrap(),
                    if contains_file {
                        "contains file!"
                    } else {
                        "not found!"
                    }
                );
            }

            if contains_file {
                if self.logging_enable {
                    bios_println!("\nFound '{}'. Mounting this to root!\n", filename);
                }

                return Ok(FileSystem::<DiskType, MountedRoot> {
                    current_filesystems: self.current_filesystems,
                    root: *filesystems,
                    attached_disk: self.attached_disk.clone(),
                    logging_enable: self.logging_enable,
                    state: PhantomData::<MountedRoot>,
                });
            }
        }

        Err(BootloaderError::NoValid)
    }
}

impl<DiskType: DiskMedia> FileSystem<DiskType, MountedRoot> {
    /// # Safety
    /// This function does not check how large the buffer is, and trusts that the caller
    /// does not load a file bigger then the given buffer. This can lead to serious issues
    /// if this buffer is not checked, so its recommended that this function not be used.
    pub unsafe fn read_file_into_ptr(
        &self,
        buffer: *mut u8,
        filename: &str,
    ) -> Result<(), BootloaderError> {
        let root_fs = self.root;

        if self.logging_enable {
            bios_print!("Reading '{}'...", filename);
        }

        root_fs.load_file_to_ptr(&self.attached_disk, filename, buffer)?;

        if self.logging_enable {
            bios_println!("Done");
        }

        Ok(())
    }

    pub fn get_filesize_bytes(&self, filename: &str) -> Result<usize, BootloaderError> {
        self.root.get_filesize_bytes(&self.attached_disk, filename)
    }

    pub fn load_file_into_slice<'a>(
        &self,
        slice: &'a mut [u8],
        filename: &str,
    ) -> Result<(), BootloaderError> {
        let slice_size = slice.len();
        let filesize_bytes = self.get_filesize_bytes(filename)?;

        if slice_size <= filesize_bytes {
            return Err(BootloaderError::OutOfBounds);
        }

        unsafe {
            self.read_file_into_ptr(slice.as_mut_ptr(), filename)?;
        }

        Ok(())
    }
}
