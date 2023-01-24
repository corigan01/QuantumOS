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
furnished to do so, subject to the following conditions

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

use core::fmt::{Debug, Formatter};
use crate::{bitset, enum_iterator};
use crate::bitset::BitSet;
use crate::error_utils::QuantumError;
use crate::memory::VirtualAddress;
use crate::enum_iterator::EnumIntoIterator;

enum_iterator! {
    #[repr(u8)]
    #[derive(Copy, Clone, PartialEq)]
    pub enum EntryType(EntryTypeIter) {
        EntryPageMapLevel5 = 0b000,
        EntryPageMapLevel4 = 0b001,
        EntryPageMapLevel3 = 0b010,
        EntryPageMapLevel2 = 0b011,
        EntryPageMapLevel1 = 0b100,
        PageEntryLevel2    = 0b101,
        PageEntryLevel1    = 0b110,
        PageEntryLevel0    = 0b111
    }
}

enum_iterator! {
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub enum PageEntryOptions(PageEntryOptionsIter) {
        Present,
        Writable,
        UserAccessible,
        WriteThrough,
        Accessed,
        NoCache,
        NoExecute,
        PageSize,
        PageAttributeTable,
        Dirty,
        Global,
        ProtectionKeyBit0,
        ProtectionKeyBit1,
        ProtectionKeyBit2,
        ProtectionKeyBit3
    }
}

impl EntryType {
    pub fn from_u64(num: u64) -> Option<Self> {
        match num {
            0 => Some(EntryType::EntryPageMapLevel5),
            1 => Some(EntryType::EntryPageMapLevel4),
            2 => Some(EntryType::EntryPageMapLevel3),
            3 => Some(EntryType::EntryPageMapLevel2),
            4 => Some(EntryType::EntryPageMapLevel1),
            5 => Some(EntryType::PageEntryLevel2),
            6 => Some(EntryType::PageEntryLevel1),
            7 => Some(EntryType::PageEntryLevel0),
            _ => None
        }
    }

    pub fn is_table(&self) -> bool {
        (*self as u8) <= 4
    }

    pub fn is_entry(&self) -> bool {
        !self.is_table()
    }
}

macro_rules! shift {
    ($epr:literal) => {
        (1_u64 << ($epr as u64)) as u64
    };
}

macro_rules! all_same {
    ($epr:expr) => { [$epr, $epr, $epr, $epr, $epr, $epr, $epr] };
}

macro_rules! zero_tables {
    ($epr:expr) => { [0_u64, 0_u64, 0_u64, 0_u64, $epr, $epr, $epr] };
}

impl PageEntryOptions {
    // [0, 0, 0, 0,  0,  0,  0 ]
    //  4  3  2  1  =2= =1= =0=
    //
    // ------------ Table ----------------
    // 4: Page Map Level 5 Entry
    // 3: Page Map Level 4 Entry
    // 2: Page Directory Pointer Table Entry
    // 1: Page Directory Entry
    // ------------ Entry ---------------
    //=2: Page Directory Pointer Table Entry
    //=1: Page Directory Entry
    //=0: Page Table Entry

    const CONV_TABLE: [(PageEntryOptions, [u64; 7]); 15] = [
        (PageEntryOptions::Present, all_same!(shift!(0))),
        (PageEntryOptions::Writable, all_same!(shift!(1))),
        (PageEntryOptions::UserAccessible, all_same!(shift!(2))),
        (PageEntryOptions::WriteThrough, all_same!(shift!(3))),
        (PageEntryOptions::NoCache, all_same!(shift!(4))),
        (PageEntryOptions::Accessed, all_same!(shift!(5))),
        (PageEntryOptions::Dirty, zero_tables!(shift!(6))),
        (PageEntryOptions::PageSize, [0_u64, 0_u64, shift!(7), shift!(7), shift!(7), shift!(7), 0_u64]),
        (PageEntryOptions::PageAttributeTable, [0_u64, 0_u64, 0_u64, 0_u64, shift!(12), shift!(12), shift!(7)]),
        (PageEntryOptions::Global, zero_tables!(shift!(1))),
        (PageEntryOptions::ProtectionKeyBit0, zero_tables!(shift!(59))),
        (PageEntryOptions::ProtectionKeyBit1, zero_tables!(shift!(60))),
        (PageEntryOptions::ProtectionKeyBit2, zero_tables!(shift!(61))),
        (PageEntryOptions::ProtectionKeyBit3, zero_tables!(shift!(62))),
        (PageEntryOptions::NoExecute, all_same!(shift!(63))),
    ];

    pub fn to_u64(self, entry_type: EntryType) -> Option<u64> {
        for (option, values) in &Self::CONV_TABLE {
            if *option == self {
                let entry_idx = entry_type as usize;
                let value = values[entry_idx];

                if value > 0 {
                    return Some(value);
                }
            }
        }

        None
    }

    pub fn from_64(entry_type: EntryType, num: u64) -> Option<Self> {
        if num == 0 {
            return None;
        }

        for (option, values) in &Self::CONV_TABLE {
            let entry_idx = entry_type as usize;
            let value = values[entry_idx];

            if value == num {
                return Some(*option);
            }
        }

        None
    }
}

