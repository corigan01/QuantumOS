/*!
```text
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
```
*/

use crate::bitset::BitSet;
use crate::error_utils::QuantumError;
use crate::memory::paging::page_flags::{EntryType, PageEntryOptions};
use crate::memory::VirtualAddress;

#[derive(Debug, Clone, Copy)]
struct RawPageEntry(u64);

impl RawPageEntry {
    pub fn new(entry_type: EntryType) -> Self {
        let mut new = Self::empty();
        new.set_entry_type(entry_type);

        new
    }

    pub fn empty() -> Self {
        Self{
            0: 0
        }
    }

    pub fn contains_next_table(&self) -> bool {
        self.0.get_bits(13..51) > 0
    }

    pub unsafe fn get_address(&self) -> Option<VirtualAddress> {
        if !self.contains_next_table() {
            return None;
        }

        let mut ptr: u64 = match self.get_entry_type() {
            EntryType::PageEntryLevel2    => { self.0.get_bits(13..51) << 13 },
            EntryType::PageEntryLevel1    => { self.0.get_bits(13..51) << 13 },
            _ => { self.0.get_bits(12..51) << 12 }
        };

        Some(VirtualAddress::new(ptr))
    }

    pub unsafe fn set_address(&mut self, address: VirtualAddress) -> Result<(), QuantumError> {
        if !address.is_aligned() {
            return Err(QuantumError::NotAligned);
        }

        let addr: u64 = address.as_u64() >> 12;

        match self.get_entry_type() {
            EntryType::PageEntryLevel2    => {
                if addr.get_bits(13..29) > 0 {
                    return Err(QuantumError::NotAligned);
                }

                self.0.set_bits(12..51, addr);
            },
            EntryType::PageEntryLevel1    => {
                if addr.get_bits(13..20) > 0 {
                    return Err(QuantumError::NotAligned);
                }

                self.0.set_bits(12..51, addr);
            },
            _ => { self.0.set_bits(12..51, addr); }
        }

        Ok(())
    }

    pub fn enable_option(&mut self, option: PageEntryOptions) -> Result<(), QuantumError> {
        if let Some(val) = option.to_u64(self.get_entry_type()) {
            self.0 |= val;
            return Ok(());
        }

        Err(QuantumError::NotValid)
    }

    pub fn disable_option(&mut self, option: PageEntryOptions) -> Result<(), QuantumError> {
        if let Some(val) = option.to_u64(self.get_entry_type()) {
            self.0 &= !(val);
            return Ok(());
        }

        Err(QuantumError::NotValid)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn reset_entry_to_new_type(&mut self, entry_type: EntryType) {
        self.0 = 0;

        self.set_entry_type(entry_type);
    }

    pub fn get_entry_type(&self) -> EntryType {
        EntryType::from_u64(self.0.get_bits(9..11)).unwrap()
    }

    fn set_entry_type(&mut self, entry_type: EntryType) {
        self.0.set_bits(9..11, entry_type as u64);
    }
}