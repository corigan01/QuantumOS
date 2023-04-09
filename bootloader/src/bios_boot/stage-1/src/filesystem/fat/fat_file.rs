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
use quantum_lib::heapless_string::HeaplessString;

pub struct FatFile {
    pub filename: HeaplessString<32>,
    pub start_cluster: usize,
    pub filesize_bytes: usize,
    pub filetype: FatFileType,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
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
    pub fn to_fat_file(&self) -> Result<FatFile, BootloaderError> {
        let cstring = HeaplessString::from_bytes(&self.file_name).unwrap();
        let starting_cluster = self.low_entry_bytes;

        let file_type = FatFileType::new_from_file(self);

        Ok(FatFile {
            filename: cstring,
            start_cluster: starting_cluster as usize,
            filesize_bytes: self.file_bytes as usize,
            filetype: file_type,
        })
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct FatLongFileName {
    pub sq_order: u8,
    pub first_5: [u16; 5],
    pub file_attributes: u8,
    pub long_entry_type: u8,
    pub checksum: u8,
    pub next_6: [u16; 6],
    pub reserved: u16,
    pub final_2: [u16; 2],
}

impl FatLongFileName {
    unsafe fn paste_name_into_buffer(&self, buffer: &mut [u8], offset: usize) {
        // FIXME: This is stupid, but I cant think of another way to do this
        for i in 0..5 {
            buffer[i + offset] = self.first_5[i] as u8;
        }
        for i in 0..6 {
            buffer[i + 5 + offset] = self.next_6[i] as u8;
        }
        for i in 0..2 {
            buffer[i + 5 + 6 + offset] = self.final_2[i] as u8;
        }
    }

    pub unsafe fn accumulate_name(&self, buffer: &mut [u8]) {
        // FIXME: This is stupid! :(
        let mut tmp_buffer = [0_u8; 256];
        self.paste_name_into_buffer(&mut tmp_buffer, 0);

        let mut data_len = 0;
        for i in 0..tmp_buffer.len() {
            if tmp_buffer[i] == 0 {
                data_len = i;
                break;
            }
        }

        let mut moved_buffer_data = [0_u8; 256];
        for i in 0..data_len {
            moved_buffer_data[i + data_len] = buffer[i];
        }
        for i in 0..data_len {
            moved_buffer_data[i] = tmp_buffer[i];
        }

        for i in 0..buffer.len() {
            buffer[i] = moved_buffer_data[i];
        }
    }
}

#[derive(PartialEq)]
pub enum FatFileType {
    ReadOnly,
    Hidden,
    System,
    LongFileName,
    VolumeLabel,
    Directory,
    File,

    Root,
    Unknown(u8),
    Zero,
}

impl FatFileType {
    pub fn new_from_file(entry: &FatDirectoryEntry) -> Self {
        let attr = entry.file_attributes;

        let expected_type = match attr {
            0x00 => Self::Zero,
            0x01 => Self::ReadOnly,
            0x02 => Self::Hidden,
            0x04 => Self::System,
            0x08 => Self::VolumeLabel,
            0x0f => Self::LongFileName,
            0x10 => Self::Directory,
            0x20 => Self::File,

            _ => Self::Unknown(attr),
        };

        // This is not part of the spec, but it seems that sometimes
        // we detect a file as being zero when its a fully valid file,
        // so this is a way of still reading malformed/out-of-spec files
        if expected_type == Self::Zero && entry.file_name[0] != 0 {
            Self::File
        } else {
            expected_type
        }
    }
}
