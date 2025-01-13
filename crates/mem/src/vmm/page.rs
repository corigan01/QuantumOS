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
use core::fmt::Display;

use alloc::boxed::Box;
use alloc::sync::Arc;
use arch::paging64::{
    PageEntry1G, PageEntry2M, PageEntry4K, PageEntryLvl2, PageEntryLvl3, PageEntryLvl4,
    PageMapLvl1, PageMapLvl2, PageMapLvl3, PageMapLvl4,
};
use lldebug::{logln, sync::Mutex};
use spin::RwLock;
use util::is_align_to;

use crate::{MemoryError, pmm::PhysPage};

use super::VirtPage;

// FIXME: Make a data structure that makes more sense for this
static CURRENT_PAGE_TABLE_ALLOC: Mutex<Option<VmSafePageTable>> = Mutex::new(None);

/// Convert a Virtal Address to a Physical Address
pub fn virt_to_phys(virt: u64) -> Option<u64> {
    let page_table = CURRENT_PAGE_TABLE_ALLOC.lock();

    if let Some(safe_table) = page_table.as_ref() {
        safe_table.lookup_virt_address(virt)
    } else {
        // FIXME: This is just hardcoded for now, but should be populated from the bootloader!!!!
        //
        // Since we know the bootloader is identity mapped, the physical PTRs are valid virtual PTRs!
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
                    lvl3_entry.get_next_entry_phy_address()
                        + (virt & (PageMapLvl3::SIZE_PER_INDEX - 1)),
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
                    lvl2_entry.get_next_entry_phy_address()
                        + (virt & (PageMapLvl2::SIZE_PER_INDEX - 1)),
                );
            }

            let lvl1_table_ptr = lvl2_entry.get_next_entry_phy_address() as *const PageMapLvl1;
            let lvl1_entry = (&*lvl1_table_ptr).get(lvl1_idx);

            if !lvl1_entry.is_present_set() {
                return None;
            }

            Some(lvl1_entry.get_phy_address() + (virt & (PageMapLvl1::SIZE_PER_INDEX - 1)))
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

    pub fn set_raw(
        &mut self,
        index: usize,
        entry: PageEntryLvl4,
        table: Option<Box<VmSafePageLvl3>>,
    ) {
        self.phys_table.store(entry, index);
        self.vm_table[index] = table;
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

    pub fn set_raw_table(
        &mut self,
        index: usize,
        entry: PageEntryLvl3,
        table: Option<Box<VmSafePageLvl2>>,
    ) {
        self.phys_table.store(entry, index);
        if let Some(table) = table {
            self.vm_table[index] = NextTableKind::Table(table);
        }
    }

    pub fn set_raw_entry(&mut self, index: usize, entry: PageEntry1G) {
        self.phys_table.store(entry, index);
        self.vm_table[index] = NextTableKind::LargePage;
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

    pub fn set_raw_table(
        &mut self,
        index: usize,
        entry: PageEntryLvl2,
        table: Option<Box<VmSafePageLvl1>>,
    ) {
        self.phys_table.store(entry, index);
        if let Some(table) = table {
            self.vm_table[index] = NextTableKind::Table(table);
        }
    }

    pub fn set_raw_entry(&mut self, index: usize, entry: PageEntry2M) {
        self.phys_table.store(entry, index);
        self.vm_table[index] = NextTableKind::LargePage;
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

    pub fn set_raw(&mut self, index: usize, entry: PageEntry4K) {
        self.phys_table.store(entry, index);
    }
}

#[derive(Clone)]
pub struct VmSafePageTable {
    cr3: Arc<RwLock<VmSafePageLvl4>>,
}

impl VmSafePageTable {
    pub fn new() -> Self {
        Self {
            cr3: Arc::new(RwLock::new(VmSafePageLvl4::new())),
        }
    }

    /// Copies the page table mapping from the bootloader.
    pub fn copy_from_bootloader() -> Self {
        let lvl4 = Arc::new(RwLock::new(VmSafePageLvl4::new()));

        let lvl4_table_ptr =
            arch::registers::cr3::get_page_directory_base_register() as *const PageMapLvl4;

        unsafe {
            // Yeah... okay maybe don't do this :)
            (&*lvl4_table_ptr)
                .entry_iter()
                .enumerate()
                .for_each(|(lvl4_index, lvl4_entry)| {
                    lvl4.write().set_raw(
                        lvl4_index,
                        lvl4_entry,
                        lvl4_entry.get_table().map(|lvl3| {
                            let mut safe_lvl3 = Box::new(VmSafePageLvl3::new());
                            lvl3.entry_iter()
                                .enumerate()
                                .for_each(|(lvl3_index, lvl3_entry)| {
                                    // BIG
                                    if let Some(lvl3_large_entry) = PageEntry1G::convert_entry(lvl3_entry) {
                                        safe_lvl3.set_raw_entry(lvl3_index, lvl3_large_entry);
                                    }
                                    // TABLE
                                    else if lvl3_entry.is_present_set() {
                                        safe_lvl3.set_raw_table(lvl3_index, lvl3_entry, lvl3_entry.get_table().map(|lvl2| {
                                            let mut safe_lvl2 = Box::new(VmSafePageLvl2::new());

                                            lvl2.entry_iter().enumerate().for_each(|(lvl2_index, lvl2_entry)| {
                                                // BIG
                                                if let Some(lvl2_large_entry) = PageEntry2M::convert_entry(lvl2_entry) {
                                                    safe_lvl2.set_raw_entry(lvl2_index, lvl2_large_entry);
                                                }
                                                // TABLE
                                                else if lvl2_entry.is_present_set(){
                                                    safe_lvl2.set_raw_table(lvl2_index, lvl2_entry, lvl2_entry.get_table().map(|lvl1| {
                                                        let mut safe_lvl1 = Box::new(VmSafePageLvl1::new());
                                                        lvl1.entry_iter().enumerate().for_each(|(lvl1_index, lvl1_entry)| {
                                                            safe_lvl1.set_raw(lvl1_index, lvl1_entry);
                                                        });

                                                        safe_lvl1
                                                    }));
                                                }
                                            });

                                            safe_lvl2
                                        }));
                                    }
                                });

                            safe_lvl3
                        }),
                    );
                });
        }

        Self { cr3: lvl4 }
    }

    /// Load these page tables into the CPU's MMU.
    ///
    /// These page tables will now be used by the system, and all future lookups for `virt_to_phys`.
    pub unsafe fn load(&self) {
        let vm_table_ptr = self.cr3.read().phys_table.table_ptr();
        let phys_table_ptr =
            virt_to_phys(vm_table_ptr).expect("Cannot get physical address for page tables!");

        *CURRENT_PAGE_TABLE_ALLOC.lock() = Some(self.clone());

        assert!(
            is_align_to(vm_table_ptr, 4096) && is_align_to(phys_table_ptr, 4096),
            "Page tables are not aligned!"
        );
        logln!("{:#016x}", phys_table_ptr);

        unsafe { arch::registers::cr3::set_page_directory_base_register(phys_table_ptr) };
        logln!("Loaded!");
    }

    /// Attempts to return the Physical Address for the given Virt Address.
    pub fn lookup_virt_address(&self, virt: u64) -> Option<u64> {
        let (lvl4_idx, lvl3_idx, lvl2_idx, lvl1_idx) = table_indexes_for(virt);

        let cr3_locked = self.cr3.read();
        let lvl3 = cr3_locked.vm_table[lvl4_idx].as_ref()?;
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

    pub fn is_loaded(&self) -> bool {
        CURRENT_PAGE_TABLE_ALLOC
            .lock()
            .as_ref()
            .is_some_and(|inner| inner.cr3.as_mut_ptr() == self.cr3.as_mut_ptr())
    }

    /// Maps the PhysPage to the VirtPage.
    pub fn map_page(
        &mut self,
        virt_page: VirtPage,
        phys_page: PhysPage,
    ) -> Result<(), MemoryError> {
        todo!()
    }
}

impl Display for VmSafePageTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let cr3_lock = self.cr3.read();

        write!(
            f,
            "PageLvl4 ({})\n",
            self.is_loaded().then_some("Loaded").unwrap_or("Unloaded")
        )?;

        for (lvl4_index, lvl4_vm) in cr3_lock.vm_table.iter().enumerate() {
            let Some(lvl4_vm) = lvl4_vm else {
                continue;
            };

            let raw = cr3_lock.phys_table.get(lvl4_index);
            write!(
                f,
                " |-PageLvl3[{:03}]: {}{}{} ({})\n",
                lvl4_index,
                raw.is_execute_disable_set().then_some("_").unwrap_or("X"),
                raw.is_read_write_set().then_some("W").unwrap_or("R"),
                raw.is_accessed_set().then_some("D").unwrap_or("_"),
                raw.is_user_access_set()
                    .then_some("User")
                    .unwrap_or("System"),
            )?;

            for (lvl3_index, lvl3_vm) in lvl4_vm.vm_table.iter().enumerate() {
                let raw = lvl4_vm.phys_table.get(lvl3_index);
                match lvl3_vm {
                    NextTableKind::NotPresent => (),
                    NextTableKind::Table(lvl2_table) => {
                        write!(
                            f,
                            " |  |-PageLvl2[{:03}]: {}{}{} ({})\n",
                            lvl3_index,
                            raw.is_execute_disable_set().then_some("_").unwrap_or("X"),
                            raw.is_read_write_set().then_some("W").unwrap_or("R"),
                            raw.is_accessed_set().then_some("D").unwrap_or("_"),
                            raw.is_user_access_set()
                                .then_some("User")
                                .unwrap_or("System"),
                        )?;

                        for (lvl2_index, lvl2_vm) in lvl2_table.vm_table.iter().enumerate() {
                            let raw = lvl2_table.phys_table.get(lvl2_index);
                            match lvl2_vm {
                                NextTableKind::NotPresent => (),
                                NextTableKind::Table(lvl1_table) => {
                                    write!(
                                        f,
                                        " |  |  |-PageLvl1[{:03}]: {}{}{} ({})\n",
                                        lvl2_index,
                                        raw.is_execute_disable_set().then_some("_").unwrap_or("X"),
                                        raw.is_read_write_set().then_some("W").unwrap_or("R"),
                                        raw.is_accessed_set().then_some("D").unwrap_or("_"),
                                        raw.is_user_access_set()
                                            .then_some("User")
                                            .unwrap_or("System"),
                                    )?;
                                    for (lvl1_index, lvl1_vm) in
                                        lvl1_table.phys_table.entry_iter().enumerate()
                                    {
                                        if lvl1_vm.is_present_set() {
                                            write!(f, " |  |  |  |-Page 4K[{:03}]\n", lvl1_index)?;
                                        }
                                    }
                                }
                                NextTableKind::LargePage => write!(
                                    f,
                                    " |  |  |-Page 2M[{:03}]: {}{}{} ({})\n",
                                    lvl2_index,
                                    raw.is_execute_disable_set().then_some("_").unwrap_or("X"),
                                    raw.is_read_write_set().then_some("W").unwrap_or("R"),
                                    raw.is_accessed_set().then_some("D").unwrap_or("_"),
                                    raw.is_user_access_set()
                                        .then_some("User")
                                        .unwrap_or("System"),
                                )?,
                            }
                        }
                    }
                    NextTableKind::LargePage => write!(
                        f,
                        " |  |-Page 1G[{:03}]: {}{}{} ({})\n",
                        lvl3_index,
                        raw.is_execute_disable_set().then_some("_").unwrap_or("X"),
                        raw.is_read_write_set().then_some("W").unwrap_or("R"),
                        raw.is_accessed_set().then_some("D").unwrap_or("_"),
                        raw.is_user_access_set()
                            .then_some("User")
                            .unwrap_or("System"),
                    )?,
                }
            }
        }

        Ok(())
    }
}
