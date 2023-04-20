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

use crate::address_utils::virtual_address::{Aligned, VirtAddress};
use crate::x86_64::paging::PagingErr;

#[derive(Debug)]
pub struct PageMapLevel4 {
    entries: InternalPageEntries,
} // Not able to be mapped

#[derive(Debug)]
pub struct PageMapLevel3 {
    entries: InternalPageEntries,
} // 1gb able to be mapped

#[derive(Debug)]
pub struct PageMapLevel2 {
    entries: InternalPageEntries,
} // 2mb able to be mapped

#[derive(Debug)]
pub struct PageMapLevel1 {
    entries: InternalPageEntries,
} // 4kb able to be mapped

#[derive(Debug)]
pub struct PageMapLevel4Entry(pub(crate) u64);

#[derive(Debug)]
pub struct PageMapLevel3Entry(pub(crate) u64);

#[derive(Debug)]
pub struct PageMapLevel2Entry(pub(crate) u64);

#[derive(Debug)]
pub struct PageMapLevel1Entry(pub(crate) u64);

impl PageMapLevel1Entry {
    pub(crate) fn add_config_options_from_u64(options: u64) -> Self {
        PageMapLevel1Entry(options)
    }
}
impl PageMapLevel2Entry {
    pub(crate) fn add_config_options_from_u64(options: u64) -> Self {
        PageMapLevel2Entry(options)
    }
}
impl PageMapLevel3Entry {
    pub(crate) fn add_config_options_from_u64(options: u64) -> Self {
        PageMapLevel3Entry(options)
    }
}
impl PageMapLevel4Entry {
    pub(crate) fn add_config_options_from_u64(options: u64) -> Self {
        PageMapLevel4Entry(options)
    }
}

impl Into<u64> for PageMapLevel1Entry {
    fn into(self) -> u64 {
        self.0
    }
}

impl Into<u64> for PageMapLevel2Entry {
    fn into(self) -> u64 {
        self.0
    }
}

impl Into<u64> for PageMapLevel3Entry {
    fn into(self) -> u64 {
        self.0
    }
}

impl Into<u64> for PageMapLevel4Entry {
    fn into(self) -> u64 {
        self.0
    }
}

#[derive(Debug)]
#[repr(align(4096), C)]
pub(crate) struct InternalPageEntries {
    entries: [u64; 512],
}

impl InternalPageEntries {
    pub fn new() -> Self {
        Self { entries: [0; 512] }
    }

    pub fn set<T>(&mut self, value: T, pos: usize) -> Result<(), PagingErr>
    where
        T: Into<u64>,
    {
        if pos > 512 {
            return Err(PagingErr::OutofBounds);
        }

        self.entries[pos] = value.into();

        Ok(())
    }

    pub fn get_u64_ptr(&self) -> u64 {
        &self.entries as *const [u64; 512] as u64
    }
}

impl PageMapLevel4 {
    pub fn new() -> Self {
        Self {
            entries: InternalPageEntries::new(),
        }
    }

    pub fn set_entry(&mut self, entry: PageMapLevel4Entry, pos: usize) -> Result<(), PagingErr> {
        self.entries.set(entry, pos)
    }

    pub fn get_entry(&self, pos: usize) -> &u64 {
        &self.entries.entries[pos]
    }

    pub fn get_address(&self) -> VirtAddress<Aligned, 12> {
        let raw_address = self.entries.get_u64_ptr();

        VirtAddress::new(raw_address)
            .unwrap()
            .try_aligned()
            .unwrap()
    }
}

impl PageMapLevel3 {
    pub fn new() -> Self {
        Self {
            entries: InternalPageEntries::new(),
        }
    }

    pub fn set_entry(&mut self, entry: PageMapLevel3Entry, pos: usize) -> Result<(), PagingErr> {
        self.entries.set(entry, pos)
    }

    pub fn get_entry(&self, pos: usize) -> &u64 {
        &self.entries.entries[pos]
    }

    pub fn get_address(&self) -> VirtAddress<Aligned, 12> {
        let raw_address = self.entries.get_u64_ptr();

        VirtAddress::new(raw_address)
            .unwrap()
            .try_aligned()
            .unwrap()
    }
}

impl PageMapLevel2 {
    pub fn new() -> Self {
        Self {
            entries: InternalPageEntries::new(),
        }
    }

    pub fn set_entry(&mut self, entry: PageMapLevel2Entry, pos: usize) -> Result<(), PagingErr> {
        self.entries.set(entry, pos)
    }

    pub fn get_entry(&self, pos: usize) -> &u64 {
        &self.entries.entries[pos]
    }

    pub fn get_address(&self) -> VirtAddress<Aligned, 12> {
        let raw_address = self.entries.get_u64_ptr();

        VirtAddress::new(raw_address)
            .unwrap()
            .try_aligned()
            .unwrap()
    }
}

impl PageMapLevel1 {
    pub fn new() -> Self {
        Self {
            entries: InternalPageEntries::new(),
        }
    }

    pub fn set_entry(&mut self, entry: PageMapLevel1Entry, pos: usize) -> Result<(), PagingErr> {
        self.entries.set(entry, pos)
    }

    pub fn get_entry(&self, pos: usize) -> &u64 {
        &self.entries.entries[pos]
    }

    pub fn get_address(&self) -> VirtAddress<Aligned, 12> {
        let raw_address = self.entries.get_u64_ptr();

        VirtAddress::new(raw_address)
            .unwrap()
            .try_aligned()
            .unwrap()
    }
}
