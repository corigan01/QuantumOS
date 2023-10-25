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

use crate::error::FsError;
use crate::filesystems::dosfs::structures::{Byte, DoubleWord, ExtendedBiosBlock, Word};
use core::mem::size_of;
use core::ptr;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ExtendedBPB32 {
    pub(crate) fat_sectors_32: DoubleWord,
    pub(crate) extended_flags: Word,
    pub(crate) filesystem_version: Word,
    pub(crate) root_cluster_number: DoubleWord,
    pub(crate) fs_info_sector: Word,
    pub(crate) backup_boot_record: Word,
    pub(crate) reserved_1: [u8; 12],
    pub(crate) drive_number: Byte,
    pub(crate) reserved_2: Byte,
    pub(crate) boot_signature: Byte,
    pub(crate) volume_serial_number: DoubleWord,
    pub(crate) volume_label: [u8; 11],
    pub(crate) filesystem_type: [u8; 8],
}

impl ExtendedBiosBlock for ExtendedBPB32 {
    fn verify(&self) -> bool {
        self.boot_signature == 0x29 && (self.volume_serial_number != 0 || self.volume_label[0] != 0)
    }

    fn volume_serial_number(&self) -> u32 {
        self.volume_serial_number
    }

    fn volume_label(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.volume_label) }
    }

    fn filesystem_string(&self) -> Option<&str> {
        Some(unsafe { core::str::from_utf8_unchecked(&self.volume_label) })
    }

    fn fat_sectors(&self) -> Option<usize> {
        Some(self.fat_sectors_32 as usize)
    }

    fn fs_info_sector(&self) -> Option<usize> {
        Some(self.fs_info_sector as usize)
    }
}

impl TryFrom<&[u8]> for ExtendedBPB32 {
    type Error = FsError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < size_of::<Self>() {
            return Err(FsError::try_from_array_error::<Self>(value));
        }

        Ok(unsafe { ptr::read(value.as_ptr() as *const Self) })
    }
}

