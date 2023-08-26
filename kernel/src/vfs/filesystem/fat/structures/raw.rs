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

use core::mem::size_of;
use core::ptr;
use qk_alloc::boxed::Box;
use qk_alloc::string::String;
use crate::vfs::filesystem::fat::structures::{FatProvider, FatType};
use crate::vfs::filesystem::FilesystemError;
use crate::vfs::io::IOError;

#[repr(C, packed)]
#[derive(Debug)]
pub struct RawBiosParameterBlock {
    pub(crate) reserved: [u8; 3],
    pub(crate) oem_id: [u8; 8],
    pub(crate) bytes_per_sector: u16,
    pub(crate) sectors_per_cluster: u8,
    pub(crate) reserved_sectors: u16,
    pub(crate) fat_tables: u8,
    pub(crate) root_dir_entries: u16,
    pub(crate) logical_sectors: u16,
    pub(crate) media_descriptor_type: u8,
    pub(crate) sectors_per_fat: u16,
    pub(crate) sectors_per_track: u16,
    pub(crate) number_of_heads: u16,
    pub(crate) hidden_sectors: u32,
    pub(crate) large_sector_count: u32
}

impl RawBiosParameterBlock {
    const RESERVED_INSTRUCTIONS: [u8; 3] = [0xEB, 0x3C, 0x90];

    pub fn is_valid(&self) -> bool {
        self.fat_tables != 0 && self.reserved == Self::RESERVED_INSTRUCTIONS
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct RawExtendedBootRecord16 {
    drive_number: u8,
    reserved: u8,
    signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    system_identifier_string: u64,
}

impl RawExtendedBootRecord16 {
    const VALID_SIG_1: u8 = 0x28;
    const VALID_SIG_2: u8 = 0x29;

    pub fn is_valid(&self) -> bool {
        self.signature == Self::VALID_SIG_1 || self.signature == Self::VALID_SIG_2
    }
}

impl FatProvider for RawExtendedBootRecord16 {
    fn fat_type(&self) -> FatType {
        FatType::Fat16
    }

    fn volume_label(&self) -> String {
        String::from(unsafe {
            core::str::from_utf8_unchecked(&self.volume_label)
        })
    }
}

impl TryFrom<&[u8]> for RawExtendedBootRecord16 {
    type Error = Box<dyn IOError>;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < size_of::<Self>() {
            return Err(Box::new(FilesystemError::Invalid));
        }

        let our_copy = unsafe { ptr::read(value.as_ptr() as *const Self) };

        if !our_copy.is_valid() {
            return Err(Box::new(FilesystemError::Invalid));
        }

        Ok(our_copy)
    }
}