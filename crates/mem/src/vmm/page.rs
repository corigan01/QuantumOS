/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

extern crate alloc;
use alloc::boxed::Box;
use alloc::sync::Arc;
use arch::paging64::{PageMapLvl1, PageMapLvl2, PageMapLvl3, PageMapLvl4};
use lldebug::sync::Mutex;

// FIXME: Make a data structure that makes more sense for this
static CURRENT_PAGE_TABLE_ALLOC: Mutex<Option<VmSafePageTable>> = Mutex::new(None);

/// Convert a Virtal Address to a Physical Address
pub fn virt_to_phys(virt: u64) -> Option<u64> {
    let page_table = CURRENT_PAGE_TABLE_ALLOC.lock();

    if let Some(safe_table) = page_table.as_ref() {
        safe_table.lookup_virt_address(virt)
    } else {
        // FIXME: This is just hardcoded for now, but should be populated from the bootloader!!!!
        let (lvl4_idx, lvl3_idx, lvl2_idx, lvl1_idx) = table_indexes_for(virt);

        let lvl4_table_ptr =
            arch::registers::cr3::get_page_directory_base_register() as *const PageMapLvl4;

        unsafe {
            let lvl4_entry = (&*lvl4_table_ptr).get(lvl4_idx);

            let lvl3_table_ptr = lvl4_entry.get_next_entry_phy_address() as *const PageMapLvl3;
            let lvl3_entry = (&*lvl3_table_ptr).get(lvl3_idx);

            if !lvl3_entry.is_present_set() {
                return None;
            }

            if lvl3_entry.is_page_size_set() {
                // 1Gib Entry
                return Some(
                    (PageMapLvl3::SIZE_PER_INDEX * lvl3_idx as u64)
                        + (PageMapLvl4::SIZE_PER_INDEX * lvl4_idx as u64),
                );
            }

            let lvl2_table_ptr = lvl3_entry.get_next_entry_phy_address() as *const PageMapLvl2;
            let lvl2_entry = (&*lvl2_table_ptr).get(lvl2_idx);

            if !lvl2_entry.is_present_set() {
                return None;
            }

            if lvl2_entry.is_page_size_set() {
                // 2Mib Entry
                return Some(
                    (PageMapLvl2::SIZE_PER_INDEX * lvl2_idx as u64)
                        + (PageMapLvl3::SIZE_PER_INDEX * lvl3_idx as u64)
                        + (PageMapLvl4::SIZE_PER_INDEX * lvl4_idx as u64),
                );
            }

            let lvl1_table_ptr = lvl2_entry.get_next_entry_phy_address() as *const PageMapLvl1;
            let lvl1_entry = (&*lvl1_table_ptr).get(lvl1_idx);

            if !lvl1_entry.is_present_set() {
                return None;
            }

            Some(lvl1_entry.get_phy_address())
        }
    }
}

#[derive(Clone)]
enum NextTableKind<T> {
    NotPresent,
    Table(Box<T>),
    LargePage,
}

/// Returns the indexes into the page tables that would give this virtual address.
const fn table_indexes_for(vaddr: u64) -> (usize, usize, usize, usize) {
    let lvl4 = PageMapLvl4::addr2index(vaddr).unwrap();
    let lvl3 = PageMapLvl3::addr2index(vaddr % PageMapLvl4::SIZE_PER_INDEX).unwrap();
    let lvl2 = PageMapLvl2::addr2index(vaddr % PageMapLvl3::SIZE_PER_INDEX).unwrap();
    let lvl1 = PageMapLvl1::addr2index(vaddr % PageMapLvl2::SIZE_PER_INDEX).unwrap();

    (lvl4, lvl3, lvl2, lvl1)
}

struct VmSafePageLvl4 {
    phys_table: PageMapLvl4,
    vm_table: [Option<Box<VmSafePageLvl3>>; 512],
}

impl VmSafePageLvl4 {
    pub const fn new() -> Self {
        Self {
            phys_table: PageMapLvl4::new(),
            vm_table: [const { None }; 512],
        }
    }
}

struct VmSafePageLvl3 {
    phys_table: PageMapLvl3,
    vm_table: [NextTableKind<VmSafePageLvl2>; 512],
}

impl VmSafePageLvl3 {
    pub const fn new() -> Self {
        Self {
            phys_table: PageMapLvl3::new(),
            vm_table: [const { NextTableKind::NotPresent }; 512],
        }
    }
}

struct VmSafePageLvl2 {
    phys_table: PageMapLvl2,
    vm_table: [NextTableKind<VmSafePageLvl1>; 512],
}

impl VmSafePageLvl2 {
    pub const fn new() -> Self {
        Self {
            phys_table: PageMapLvl2::new(),
            vm_table: [const { NextTableKind::NotPresent }; 512],
        }
    }
}

struct VmSafePageLvl1 {
    phys_table: PageMapLvl1,
}

impl VmSafePageLvl1 {
    pub const fn new() -> Self {
        Self {
            phys_table: PageMapLvl1::new(),
        }
    }
}

#[derive(Clone)]
pub struct VmSafePageTable {
    cr3: Arc<VmSafePageLvl4>,
}

impl VmSafePageTable {
    pub fn new() -> Self {
        Self {
            cr3: Arc::new(VmSafePageLvl4::new()),
        }
    }

    pub unsafe fn load(&self) {
        let mut page_tables = CURRENT_PAGE_TABLE_ALLOC.lock();
        *page_tables = Some(self.clone());

        let vm_table_ptr = self.cr3.phys_table.table_ptr();
        let phys_table_ptr =
            virt_to_phys(vm_table_ptr).expect("Cannot get physical address for page tables!");

        unsafe { arch::registers::cr3::set_page_directory_base_register(phys_table_ptr) };
    }

    /// Attempts to return the Physical Address for the given Virt Address.
    pub fn lookup_virt_address(&self, virt: u64) -> Option<u64> {
        let (lvl4_idx, lvl3_idx, lvl2_idx, lvl1_idx) = table_indexes_for(virt);

        let lvl3 = self.cr3.vm_table[lvl4_idx].as_ref()?;
        let lvl2 = match &lvl3.vm_table[lvl3_idx] {
            NextTableKind::Table(next_table) => next_table,
            NextTableKind::LargePage => {
                return Some(
                    (PageMapLvl3::SIZE_PER_INDEX * lvl3_idx as u64)
                        + (PageMapLvl4::SIZE_PER_INDEX * lvl4_idx as u64),
                );
            }
            NextTableKind::NotPresent => return None,
        };
        let lvl1 = match &lvl2.vm_table[lvl2_idx] {
            NextTableKind::Table(next_table) => next_table,
            NextTableKind::LargePage => {
                return Some(
                    (PageMapLvl2::SIZE_PER_INDEX * lvl2_idx as u64)
                        + (PageMapLvl3::SIZE_PER_INDEX * lvl3_idx as u64)
                        + (PageMapLvl4::SIZE_PER_INDEX * lvl4_idx as u64),
                );
            }
            NextTableKind::NotPresent => return None,
        };

        let page_entry = lvl1.phys_table.get(lvl1_idx);
        if !page_entry.is_present_set() {
            return None;
        }

        Some(page_entry.get_phy_address())
    }
}
