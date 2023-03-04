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
use crate::fat::FatExtCluster;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct BiosParametersBlock {
    pub jmp_bytes: [u8; 3],
    pub oem_id: u64,
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_of_fats: u8,
    pub root_entries: u16,
    pub low_sectors: u16,
    pub media_descriptor_type: u8,
    pub sectors_per_fat: u16,
    pub sectors_per_track: u16,
    pub heads_on_media: u16,
    pub hidden_sectors: u32,
    pub high_sectors: u32,
    ex_block: [u8; 54],
}

impl BiosParametersBlock {
    pub fn validate_fat(&self) -> bool {
            self.jmp_bytes[0] == 0xeb        &&
            self.bytes_per_sector == 512     &&
            self.oem_id != 0x00              &&
            self.media_descriptor_type != 0  &&
            ((self.low_sectors == 0 && self.high_sectors > 0) || self.low_sectors > 0)
    }

    pub unsafe fn get_ext_bpb<T>(&self) -> Option<&T>
        where T: FatExtCluster
    {
        let data = unsafe {
            &*(&self.ex_block as *const u8 as *const T)
        };

        if !data.is_valid_sig() {
            return None
        }

        return Some(data);

        None
    }
}