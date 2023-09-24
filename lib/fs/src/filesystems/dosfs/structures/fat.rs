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

use qk_alloc::vec::Vec;
use crate::error::{FsError, FsErrorKind};
use crate::filesystems::dosfs::structures::{MAX_SECTORS_FOR_FAT16, MAX_SECTORS_FOR_FAT32};
use crate::FsResult;

pub struct FileAllocationTable {
    data: Vec<u8>
}

impl FileAllocationTable {
    pub fn read_fat12_entry(&self, index: usize) -> FsResult<usize> {
        todo!("FAT12 fat entry")
    }

    pub fn read_fat16_entry(&self, index: usize) -> FsResult<usize> {
        if index >= MAX_SECTORS_FOR_FAT16 {
            return Err(FsError::new(FsErrorKind::InvalidInput,
                                    "Cannot address more then 65525 clusters in fat16"));
        }

        let byte_offset = index * 2;

        Ok(((self.data[byte_offset] as u16) << 8 | (self.data[byte_offset + 1] as u16)) as usize)
    }

    pub fn read_fat32_entry(&self, index: usize) -> FsResult<usize> {
        if index >= MAX_SECTORS_FOR_FAT32 {
            return Err(FsError::new(FsErrorKind::InvalidInput,
                                    "Cannot address more then 268435447 clusters in fat32"));
        }

        let byte_offset = index * 4;

        let mut byte_slice = [0; 4];
        byte_slice.copy_from_slice(&self.data.as_slice()[byte_offset..byte_offset + 4]);
        let large_value = u32::from_le_bytes(byte_slice);

        Ok(large_value as usize)
    }
}

impl From<&[u8]> for FileAllocationTable {
    fn from(value: &[u8]) -> Self {
        FileAllocationTable {
            data: value.into()
        }
    }
}
