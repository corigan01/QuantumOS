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

use crate::cstring::CStringRef;
use super::FatExtCluster;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Extended32 {
    pub sectors_per_fat: u32,
    pub flags: u16,
    pub fat_version: u16,
    pub root_cluster_number: u32,
    pub fs_info_structure: u16,
    pub backup_boot_sector: u16,
    reserved: [u8; 12],
    pub drive_number: u8,
    pub win_nt_flags: u8,
    pub signature: u8,
    pub vol_id: u32,
    pub vol_label: [u8; 11],
    system_id_string_dont_trust: u64,
}

impl Extended32 {

}

impl FatExtCluster for Extended32 {
    fn is_valid_sig(&self) -> bool {
        self.signature == 0x28 || self.signature == 0x29
    }

    fn get_vol_string(&self) -> Option<CStringRef> {
        Some(CStringRef::from_bytes(&self.vol_label))
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DirectoryEntry32 {
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
    pub file_bytes: u32
}

