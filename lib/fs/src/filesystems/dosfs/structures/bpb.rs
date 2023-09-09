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

use crate::filesystems::dosfs::structures::{Byte, DoubleWord, Word};

pub struct BiosParameterBlock {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: Word,
    sectors_per_cluster: Byte,
    reserved_sector_count: Word,
    number_of_fats: Byte,
    root_entry_count: Word,
    total_sectors_16: Word,
    media: Byte,
    fat_sectors_16: Word,
    sectors_per_track: Word,
    number_of_heads: Word,
    hidden_sectors: DoubleWord,
    total_sectors_32: DoubleWord
}

impl BiosParameterBlock {
    pub fn verify_jmp_instruction(&self) -> bool {
        (self.jmp_boot[0] == 0xEB && self.jmp_boot[2] == 0x90) ||
            (self.jmp_boot[0] == 0xE9)
    }

    pub fn oem_name(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.oem_name) }
    }

    pub fn verify_sector_count_correctness(&self) -> bool {
        (self.total_sectors_16 == 0 && self.total_sectors_32 > 0) &&
            (self.total_sectors_32 == 0 && self.total_sectors_16 > 0)
    }

    pub fn sectors(&self) -> usize {
        if self.total_sectors_16 == 0 {
            self.total_sectors_32 as usize
        } else {
            self.total_sectors_16 as usize
        }
    }

    pub fn sectors_occupied_by_fat16(&self) -> usize {
        self.fat_sectors_16 as usize
    }
}