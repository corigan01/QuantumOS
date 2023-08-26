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



use core::error::Error;
use core::fmt::{Debug, Display, Formatter};
use qk_alloc::boxed::Box;
use quantum_lib::debug_println;
use crate::vfs::filesystem::fat::{init_fat_fs, is_media_fat_formatted};
use crate::vfs::{VFSDiskID, VFSPartitionID};
use crate::vfs::io::{ErrorKind, IOError, IOResult};

pub mod ext2;
pub mod fat;

#[derive(Clone, Copy, Debug)]
pub enum SupportedFilesystem {
    Fat,
    Ext2
}

impl Error for FilesystemError {}

impl Display for FilesystemError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl IOError for FilesystemError {
    fn error_kind(&self) -> ErrorKind {
        match self {
            _ => ErrorKind::Unknown
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum FilesystemError {
    NotSupported,
    Invalid,
    OutOfBounds
}

pub fn detect_filesystem_in_partition(partition: &VFSPartitionID) -> IOResult<SupportedFilesystem> {
    let partition_ref = &mut partition.get_entry_mut().partition;

    if is_media_fat_formatted(partition_ref)? {
        init_fat_fs(*partition).unwrap();
        Ok(SupportedFilesystem::Fat)
    } else {
        Err(Box::new(FilesystemError::NotSupported))
    }
}

pub fn try_init_on_all() {
    for disk in VFSDiskID::disks_iter() {
        for part in disk.parts.iter() {
            let filesystem = detect_filesystem_in_partition(&part.id);

            if let Err(e) = filesystem {
                debug_println!("Filesystem Reported {:#?}! Skipping ... ", e);
                continue;
            }

            debug_println!("New Filesystem found {:#?}", filesystem);
        }
    }
}