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

use crate::cstring::CStringOwned;

pub struct FatFile {
    pub filename: CStringOwned,
    pub start_cluster: usize,
    pub filesize_bytes: usize,
    pub filetype: FatFileType
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct FatDirectoryEntry {
    pub file_name: [u8; 8],
    pub file_extension: [u8; 3],
    pub file_attributes: u8,
    reserved_win_nt: u8,
    pub creation_time_tens_of_second: u8,
    pub creation_time: u16,
    pub creation_date: u16,
    pub last_accessed_byte: u16,
    pub high_entry_bytes: u16,
    pub modification_time: u16,
    pub modification_date: u16,
    pub low_entry_bytes: u16,
    pub file_bytes: u32,
}

impl FatDirectoryEntry {
    pub fn to_fat_file(&self) -> FatFile {
        let filename = self.file_name.as_ptr();
        let cstring = unsafe {
            CStringOwned::from_ptr(filename, 11)
        };
        let starting_cluster = self.low_entry_bytes;

        let file_type = FatFileType::new_from_file(self);

        FatFile {
            filename: cstring,
            start_cluster: starting_cluster as usize,
            filesize_bytes: self.file_bytes as usize,
            filetype: file_type,
        }
    }
}

#[derive(Debug)]
pub enum FatFileType {
    ReadOnly,
    Hidden,
    System,
    LongFileName,
    VolumeLabel,
    Directory,
    File,
    
    Root,
    Unknown,
}

impl FatFileType {
    pub fn new_from_file(entry: &FatDirectoryEntry) -> Self {
        let attr = entry.file_attributes;

        match attr {
            0x01 => Self::ReadOnly,
            0x02 => Self::Hidden,
            0x04 => Self::System,
            0x08 => Self::VolumeLabel,
            0x0f => Self::LongFileName,
            0x10 => Self::Directory,
            0x20 => Self::File,

            _ => Self::Unknown,
        }
    }
}