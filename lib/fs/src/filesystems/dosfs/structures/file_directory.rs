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

use core::mem::size_of;
use core::ptr;
use qk_alloc::string::String;
use crate::error::{FsError};
use crate::filesystems::dosfs::structures::{Byte, ClusterID, DoubleWord, FatTime, Word};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DirectoryEntryAttributes {
    Unknown, // 0x00
    ReadOnly, // 0x01
    Hidden, // 0x02
    System, // 0x04
    VolumeID, // 0x08
    Directory, // 0x10
    Archive, // 0x20
    LongName // or all of the above
}

impl From<Byte> for DirectoryEntryAttributes {
    fn from(value: Byte) -> Self {
        match value {
            0x01 => Self::ReadOnly,
            0x02 => Self::Hidden,
            0x04 => Self::System,
            0x08 => Self::VolumeID,
            0x10 => Self::Directory,
            0x20 => Self::Archive,
            0x3F => Self::LongName,

            _ => Self::Unknown,
        }
    }
}

impl Into<Byte> for DirectoryEntryAttributes {
    fn into(self) -> Byte {
        match self {
            Self::Unknown => 0x00,
            Self::ReadOnly => 0x01,
            Self::Hidden => 0x02,
            Self::System => 0x04,
            Self::VolumeID => 0x08,
            Self::Directory => 0x10,
            Self::Archive => 0x20,
            Self::LongName => 0x3F,
        }
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct DirectoryEntry {
    name: [u8; 11],
    attribute_types: Byte,
    reserved: Byte,
    creation_time_tenth: Byte,
    creation_time: Word,
    creation_date: Word,
    last_access_date: Word,
    first_data_cluster_high: Word,
    last_modification_time: Word,
    last_modification_date: Word,
    first_data_cluster_low: Word,
    file_size: DoubleWord
}

impl TryFrom<&[u8]> for DirectoryEntry {
    type Error = FsError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < size_of::<Self>() {
            return Err(FsError::try_from_array_error::<Self>(value));
        }

         Ok(unsafe { ptr::read(value.as_ptr() as *const Self) })
    }
}

impl DirectoryEntry {
    pub fn is_entry_free(&self) -> bool {
        self.name[0] == 0xE5 || self.name[0] == 0
    }

    pub fn creation_time(&self) -> FatTime {
        FatTime::from_fat_time_and_date(self.creation_date, self.creation_time)
    }

    pub fn last_access_time(&self) -> FatTime {
        FatTime::from_fat_date(self.last_access_date)
    }

    pub fn write_time(&self) -> FatTime {
        FatTime::from_fat_time_and_date(self.last_modification_date, self.last_modification_time)
    }

    pub fn first_cluster(&self) -> ClusterID {
        (self.first_data_cluster_high as ClusterID) << 16 | (self.first_data_cluster_low as ClusterID)
    }

    pub fn short_filename(&self) -> String {
        let first_part = &self.name[..=8];
        let dot_part = &self.name[9..];

        let mut building_string = String::with_capacity(11);

        for &byte in first_part {
            if byte == ' ' as u8 {
                break;
            }

            building_string.push((byte as char).to_ascii_uppercase());
        }

        if dot_part[0] != ' ' as u8 {
            building_string.push('.');
        }

        for &byte in dot_part {
            if byte == ' ' as u8 {
                break;
            }

            building_string.push((byte as char).to_ascii_uppercase());
        }

        building_string
    }

    pub fn entry_attributes(&self) -> DirectoryEntryAttributes {
        self.attribute_types.into()
    }

    pub fn file_size(&self) -> usize {
        self.file_size as usize
    }
}

#[cfg(test)]
mod test {
    use crate::filesystems::dosfs::structures::file_directory::{DirectoryEntry, DirectoryEntryAttributes};
    use crate::set_example_allocator;

    #[test]
    fn test_dir_from_kernel() {
        set_example_allocator(4096);
        let example: [u8; 0x20] = [
            0x4B, 0x45, 0x52, 0x4E, 0x45, 0x4C, 0x20, 0x20, 0x45, 0x4C, 0x46, 0x00, 0x00, 0x70, 0x04, 0x91,
            0x41, 0x57, 0x41, 0x57, 0x00, 0x00, 0x04, 0x91, 0x41, 0x57, 0x2C, 0x00, 0x70, 0xC8, 0x02, 0x00
        ];

        let file_entry = DirectoryEntry::try_from(example.as_ref()).unwrap();

        assert!(false, "{:#x?}", file_entry);



    }
}