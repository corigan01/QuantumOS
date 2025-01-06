/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2024 Gavin Kellam

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

use hw::make_hw;

/// The max 'bits' of physical memory the system supports.
pub const MAX_PHY_MEMORY_WIDTH: usize = 48;

#[make_hw(
    field(RW, 0, pub present),
    field(RW, 1, pub read_write),
    field(RW, 2, pub user_accessed),
    field(RW, 3, pub write_though),
    field(RW, 4, pub cache_disable),
    field(RW, 5, pub accessed),
    field(RW, 6, pub dirty),
    field(RW, 7, pub page_attribute_table),
    field(RW, 8, pub global),
    field(RWNS, 12..48, pub phy_address),
    field(RW, 59..62, pub protection_key),
    field(RW, 63, pub execute_disable)
)]
#[derive(Clone, Copy)]
pub struct PageEntry4K(u64);

impl PageEntry4K {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn new() -> Self {
        Self::zero()
    }
}

#[make_hw(
    field(RW, 0, pub present),
    field(RW, 1, pub read_write),
    field(RW, 2, pub user_accessed),
    field(RW, 3, pub write_though),
    field(RW, 4, pub cache_disable),
    field(RW, 5, pub accessed),
    field(RW, 6, pub dirty),
    /// For this entry, `page_size` needs to be set to true! 
    field(RW, 7, pub page_size),
    field(RW, 8, pub global),
    field(RW, 12, pub page_attribute_table),
    field(RWNS, 21..48, pub phy_address),
    field(RW, 59..62, pub protection_key),
    field(RW, 63, pub execute_disable)
)]
#[derive(Clone, Copy)]
pub struct PageEntry2M(u64);

impl PageEntry2M {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn new() -> Self {
        Self::zero().set_page_size_flag(true)
    }
}

#[make_hw(
    field(RW, 0, pub present),
    field(RW, 1, pub read_write),
    field(RW, 2, pub user_accessed),
    field(RW, 3, pub write_though),
    field(RW, 4, pub cache_disable),
    field(RW, 5, pub accessed),
    field(RW, 6, pub dirty),
    /// For this entry, `page_size` needs to be set to true! 
    field(RW, 7, pub page_size),
    field(RW, 8, pub global),
    field(RW, 12, pub page_attribute_table),
    field(RWNS, 21..48, pub phy_address),
    field(RW, 59..62, pub protection_key),
    field(RW, 63, pub execute_disable)
)]
#[derive(Clone, Copy)]
pub struct PageEntry1G(u64);

impl PageEntry1G {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn new() -> Self {
        Self::zero().set_page_size_flag(true)
    }
}

/// A Page Directory Table Entry
///
/// # How to use?
///
/// Here we are building a `PageDirectryEntry` with the `P`, `R/W`, and `U/S` bits set. The
/// bit-field in `entry` will correspond to this change (should be compiled in).
///
/// # Safety
/// This is not 'unsafe' however, its not fully 'safe' either. When loading the page
/// tables themselves, one must understand and verify that all page tables are
/// loaded correctly. Each entry in the page table isn't unsafe by itself,
/// however, when loaded into the system it becomes unsafe.
///
/// It would be a good idea to verify that all 'bit' or options set in this entry  does exactly
/// what you intend it to do before loading it. Page tables can cause the entire system to become
/// unstable if mapped wrong -- **this is very important.**
#[make_hw( 
    field(RW, 0, pub present),
    field(RW, 1, pub read_write),
    field(RW, 2, pub user_access),
    field(RW, 3, pub write_though),
    field(RW, 4, pub cache_disable),
    field(RW, 5, pub accessed),
    /// In this mode `page_size` needs to be set to false!
    field(RW, 7, pub page_size),
    field(RWNS, 12..48, pub next_entry_phy_address),
    field(RW, 63, pub execute_disable),
)]
#[derive(Clone, Copy)]
pub struct PageEntryLvl2(u64);

impl PageEntryLvl2 {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn new() -> Self {
        Self::zero()
    }
}

/// A Page Directory Pointer Table Entry
///
/// # How to use?
///
/// Here we are building a `PageDirectryEntry` with the `P`, `R/W`, and `U/S` bits set. The
/// bit-field in `entry` will correspond to this change (should be compiled in).
///
/// # Safety
/// This is not 'unsafe' however, its not fully 'safe' either. When loading the page
/// tables themselves, one must understand and verify that all page tables are
/// loaded correctly. Each entry in the page table isn't unsafe by itself,
/// however, when loaded into the system it becomes unsafe.
///
/// It would be a good idea to verify that all 'bit' or options set in this entry  does exactly
/// what you intend it to do before loading it. Page tables can cause the entire system to become
/// unstable if mapped wrong -- **this is very important.**
#[make_hw( 
    field(RW, 0, pub present),
    field(RW, 1, pub read_write),
    field(RW, 2, pub user_access),
    field(RW, 3, pub write_though),
    field(RW, 4, pub cache_disable),
    field(RW, 5, pub accessed),
    /// In this mode `page_size` needs to be set to false!
    field(RW, 7, pub page_size),
    field(RWNS, 12..48, pub next_entry_phy_address),
    field(RW, 63, pub execute_disable),
)]
#[derive(Clone, Copy)]
pub struct PageEntryLvl3(u64);

impl PageEntryLvl3 {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn new() -> Self {
        Self::zero()
    }
}

/// A Page Level 4 Table Entry
///
/// # How to use?
///
/// Here we are building a `PageDirectryEntry` with the `P`, `R/W`, and `U/S` bits set. The
/// bit-field in `entry` will correspond to this change (should be compiled in).
///
/// # Safety
/// This is not 'unsafe' however, its not fully 'safe' either. When loading the page
/// tables themselves, one must understand and verify that all page tables are
/// loaded correctly. Each entry in the page table isn't unsafe by itself,
/// however, when loaded into the system it becomes unsafe.
///
/// It would be a good idea to verify that all 'bit' or options set in this entry  does exactly
/// what you intend it to do before loading it. Page tables can cause the entire system to become
/// unstable if mapped wrong -- **this is very important.**
#[make_hw( 
    field(RW, 0, pub present),
    field(RW, 1, pub read_write),
    field(RW, 2, pub user_access),
    field(RW, 3, pub write_though),
    field(RW, 4, pub cache_disable),
    field(RW, 5, pub accessed),
    /// In this mode `page_size` needs to be set to false!
    field(RW, 7, pub page_size),
    field(RWNS, 12..48, pub next_entry_phy_address),
    field(RW, 63, pub execute_disable),
)]
#[derive(Clone, Copy)]
pub struct PageEntryLvl4(u64);

impl PageEntryLvl4 {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn new() -> Self {
        Self::zero()
    }
}

/// A Page Level 4 Table Entry
///
/// # How to use?
///
/// Here we are building a `PageDirectryEntry` with the `P`, `R/W`, and `U/S` bits set. The
/// bit-field in `entry` will correspond to this change (should be compiled in).
///
/// # Safety
/// This is not 'unsafe' however, its not fully 'safe' either. When loading the page
/// tables themselves, one must understand and verify that all page tables are
/// loaded correctly. Each entry in the page table isn't unsafe by itself,
/// however, when loaded into the system it becomes unsafe.
///
/// It would be a good idea to verify that all 'bit' or options set in this entry  does exactly
/// what you intend it to do before loading it. Page tables can cause the entire system to become
/// unstable if mapped wrong -- **this is very important.**
#[make_hw( 
    field(RW, 0, pub present),
    field(RW, 1, pub read_write),
    field(RW, 2, pub user_access),
    field(RW, 3, pub write_though),
    field(RW, 4, pub cache_disable),
    field(RW, 5, pub accessed),
    /// In this mode `page_size` needs to be set to false!
    field(RW, 7, pub page_size),
    field(RW, 12..48, pub next_entry_phy_address),
    field(RW, 63, pub execute_disable),
)]
#[derive(Clone, Copy)]
pub struct PageEntryLvl5(u64);

impl PageEntryLvl5 {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn new() -> Self {
        Self::zero()
    }
}

#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct PageMapLvl5([u64; 512]);

#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct PageMapLvl4([u64; 512]);

#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct PageMapLvl3([u64; 512]);

#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct PageMapLvl2([u64; 512]);

#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct PageMapLvl1([u64; 512]);

// TODO: Make docs for these
// Theses are the entires that can fit into the tables
pub trait Lvl5Entry {
    fn into_raw(self) -> u64;
}

impl Lvl5Entry for PageEntryLvl5 {
    fn into_raw(self) -> u64 {
        self.0
    }
}

pub trait Lvl4Entry {
    fn into_raw(self) -> u64;
}

impl Lvl4Entry for PageEntryLvl4 {
    fn into_raw(self) -> u64 {
        self.0
    }
} 

pub trait Lvl3Entry {
    fn into_raw(self) -> u64;
}

impl Lvl3Entry for PageEntryLvl3 {
    fn into_raw(self) -> u64 {
        self.0
    }
}

impl Lvl3Entry for PageEntry1G {
    fn into_raw(self) -> u64 {
        self.0
    }
}

pub trait Lvl2Entry {
    fn into_raw(self) -> u64;
}

impl Lvl2Entry for PageEntryLvl2 {
    fn into_raw(self) -> u64 {
        self.0
    }
}

impl Lvl2Entry for PageEntry2M {
    fn into_raw(self) -> u64 {
        self.0
    }
}

pub trait Lvl1Entry {
    fn into_raw(self) -> u64;
}

impl Lvl1Entry for PageEntry4K {
    fn into_raw(self) -> u64 {
        self.0
    }
}

impl PageMapLvl1 {
    pub const SIZE_PER_INDEX: u64 = util::consts::PAGE_4K as u64;
    pub const SIZE_FOR_TABLE: u64 = util::consts::PAGE_4K as u64 * 512;

    pub const fn new() -> Self {
        Self([0; 512])
    }

    /// Convert an address to a table offset.
    ///
    /// If the given `addr` is larger than the page table,
    /// it will return `None`.
    pub const fn addr2index(addr: u64) -> Option<usize> {
        if addr > Self::SIZE_FOR_TABLE {
            None
        } else {
            Some(((addr.saturating_sub(1)) / Self::SIZE_PER_INDEX) as usize + 1)
        }
    }

    pub fn store(&mut self, entry: impl Lvl1Entry, index: usize) {
        self.0[index] = entry.into_raw();
    }

    pub fn flood_table(&mut self, entry: impl Lvl1Entry) {
        self.0 = [entry.into_raw(); 512];
    }

    pub fn table_ptr(&self) -> u64 {
        self.0.as_ptr() as u64
    }
}

impl PageMapLvl2 {
    pub const SIZE_PER_INDEX: u64 = util::consts::PAGE_2M as u64;
    pub const SIZE_FOR_TABLE: u64 = util::consts::PAGE_2M as u64 * 512;

    pub const fn new() -> Self {
        Self([0; 512])
    }

    /// Convert an address to a table offset.
    ///
    /// If the given `addr` is larger than the page table,
    /// it will return `None`.
    pub const fn addr2index(addr: u64) -> Option<usize> {
        if addr > Self::SIZE_FOR_TABLE {
            None
        } else {
            Some(((addr.saturating_sub(1)) / Self::SIZE_PER_INDEX) as usize + 1)
        }
    }

    pub fn store(&mut self, entry: impl Lvl2Entry, index: usize) {
        self.0[index] = entry.into_raw();
    }

    pub fn flood_table(&mut self, entry: impl Lvl2Entry) {
        self.0 = [entry.into_raw(); 512];
    }

    pub fn table_ptr(&self) -> u64 {
        self.0.as_ptr() as u64
    }
}

impl PageMapLvl3 {
    pub const SIZE_PER_INDEX: u64 = util::consts::PAGE_1G as u64 ;
    pub const SIZE_FOR_TABLE: u64 = util::consts::PAGE_1G as u64 * 512 ;

    pub const fn new() -> Self {
        Self([0; 512])
    }
    
    /// Convert an address to a table offset.
    ///
    /// If the given `addr` is larger than the page table,
    /// it will return `None`.
    pub const fn addr2index(addr: u64) -> Option<usize> {
        if addr > Self::SIZE_FOR_TABLE {
            None
        } else {
            Some(((addr.saturating_sub(1)) / Self::SIZE_PER_INDEX) as usize + 1)
        }
    }

    pub fn store(&mut self, entry: impl Lvl3Entry, index: usize) {
        self.0[index] = entry.into_raw();
    }

    pub fn flood_table(&mut self, entry: impl Lvl3Entry) {
        self.0 = [entry.into_raw(); 512];
    }

    pub fn table_ptr(&self) -> u64 {
        self.0.as_ptr() as u64
    }
}

impl PageMapLvl4 {
    pub const SIZE_PER_INDEX: u64 = util::consts::PAGE_1G as u64 * 512;
    pub const SIZE_FOR_TABLE: u64 = util::consts::PAGE_1G as u64 * 512 * 512;

    pub const fn new() -> Self {
        Self([0; 512])
    }

    /// Convert an address to a table offset.
    ///
    /// If the given `addr` is larger than the page table,
    /// it will return `None`.
    pub const fn addr2index(addr: u64) -> Option<usize> {
        if addr > Self::SIZE_FOR_TABLE {
            None
        } else {
            Some(((addr.saturating_sub(1)) / Self::SIZE_PER_INDEX) as usize + 1)
        }
    }

    pub fn store(&mut self, entry: impl Lvl4Entry, index: usize) {
        self.0[index] = entry.into_raw();
    }

    pub fn flood_table(&mut self, entry: impl Lvl4Entry) {
        self.0 = [entry.into_raw(); 512];
    }

    pub fn table_ptr(&self) -> u64 {
        self.0.as_ptr() as u64
    }
}

impl PageMapLvl5 {
    pub const SIZE_PER_INDEX: u64 = util::consts::PAGE_1G as u64 * 512 * 512;
    pub const SIZE_FOR_TABLE: u64 = util::consts::PAGE_1G as u64 * 512 * 512 * 512;

    /// Convert an address to a table offset.
    ///
    /// If the given `addr` is larger than the page table,
    /// it will return `None`.
    pub const fn addr2index(addr: u64) -> Option<usize> {
        if addr > Self::SIZE_FOR_TABLE {
            None
        } else {
            Some(((addr.saturating_sub(1)) / Self::SIZE_PER_INDEX) as usize + 1)
        }
    }

    pub const fn new() -> Self {
        Self([0; 512])
    }

    pub fn store(&mut self, entry: impl Lvl5Entry, index: usize) {
        self.0[index] = entry.into_raw();
    }

    pub fn flood_table(&mut self, entry: impl Lvl5Entry) {
        self.0 = [entry.into_raw(); 512];
    }

    pub fn table_ptr(&self) -> u64 {
        self.0.as_ptr() as u64
    }
}

