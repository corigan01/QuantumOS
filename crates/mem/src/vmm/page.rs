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

use core::fmt::{Debug, Display};

use super::{VirtPage, VmPermissions};
use crate::{MemoryError, pmm::PhysPage};
use alloc::sync::Arc;
use arch::paging64::{
    PageEntry1G, PageEntry2M, PageEntry4K, PageEntryLvl2, PageEntryLvl3, PageEntryLvl4,
    PageMapLvl1, PageMapLvl2, PageMapLvl3, PageMapLvl4,
};
use spin::RwLock;
use util::consts::{PAGE_1G, PAGE_2M, PAGE_4K};

static CURRENTLY_LOADED_TABLE: SharedTable = SharedTable::empty();

pub fn virt_to_phys(virt: u64) -> Option<u64> {
    match CURRENTLY_LOADED_TABLE.virt_to_phys(virt) {
        Ok(virt) => Some(virt),
        Err(MemoryError::EmptySegment) | Err(MemoryError::AlreadyUsed) => {
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
        _ => None,
    }
}

/// Returns the indexes into the page tables that would give this virtual address.
const fn table_indexes_for(vaddr: u64) -> (usize, usize, usize, usize) {
    let lvl4 = PageMapLvl4::addr2index(vaddr).unwrap();
    let lvl3 = PageMapLvl3::addr2index(vaddr % PageMapLvl4::SIZE_PER_INDEX).unwrap();
    let lvl2 = PageMapLvl2::addr2index(vaddr % PageMapLvl3::SIZE_PER_INDEX).unwrap();
    let lvl1 = PageMapLvl1::addr2index(vaddr % PageMapLvl2::SIZE_PER_INDEX).unwrap();

    (lvl4, lvl3, lvl2, lvl1)
}

enum SharedState<T> {
    /// This entry is not present in the table
    NotPresent,
    /// We are the owner of this table
    OwnedTable(Arc<RwLock<T>>),
    /// We are holding a ref to this table
    RefTable(Arc<RwLock<T>>),
    /// We own this entry
    OwnedEntry,
    /// We are holding a ref to this entry
    RefEntry,
}

pub struct SharedTable {
    state: RwLock<SharedState<SharedLvl4>>,
}

pub struct SharedLvl4 {
    phys_table: PageMapLvl4,
    vm_table: [SharedState<SharedLvl3>; 512],
}

pub struct SharedLvl3 {
    phys_table: PageMapLvl3,
    vm_table: [SharedState<SharedLvl2>; 512],
}

pub struct SharedLvl2 {
    phys_table: PageMapLvl2,
    vm_table: [SharedState<SharedLvl1>; 512],
}

pub struct SharedLvl1 {
    phys_table: PageMapLvl1,
}

impl SharedTable {
    pub const fn empty() -> Self {
        Self {
            state: RwLock::new(SharedState::NotPresent),
        }
    }

    pub fn is_loaded(&self) -> bool {
        match &*self.state.read() {
            SharedState::RefTable(our_table) | SharedState::OwnedTable(our_table) => {
                match &*CURRENTLY_LOADED_TABLE.state.read() {
                    SharedState::NotPresent => false,
                    SharedState::OwnedTable(global_table) | SharedState::RefTable(global_table) => {
                        Arc::ptr_eq(our_table, global_table)
                    }
                    _ => panic!("Table does not support 'Entry' types, yet one was present!"),
                }
            }
            SharedState::NotPresent => false,
            _ => panic!("Table does not support 'Entry' types, yet one was present!"),
        }
    }

    pub fn virt_to_phys(&self, vaddr: u64) -> Result<u64, MemoryError> {
        let lvl4 = self.state.try_read().ok_or(MemoryError::AlreadyUsed)?;
        let (lvl4_index, lvl3_index, lvl2_index, lvl1_index) = table_indexes_for(vaddr);

        match &*lvl4 {
            SharedState::NotPresent => Err(MemoryError::EmptySegment),
            SharedState::OwnedTable(rw_lock) | SharedState::RefTable(rw_lock) => {
                let lvl4 = rw_lock.read();
                let shared_state = &lvl4.vm_table[lvl4_index];
                match shared_state {
                    SharedState::NotPresent => Err(MemoryError::NotFound),
                    SharedState::OwnedTable(rw_lock) | SharedState::RefTable(rw_lock) => {
                        let lvl3 = rw_lock.read();
                        let shared_state = &lvl3.vm_table[lvl3_index];
                        match shared_state {
                            SharedState::NotPresent => Err(MemoryError::NotFound),
                            SharedState::OwnedTable(rw_lock) | SharedState::RefTable(rw_lock) => {
                                let lvl2 = rw_lock.read();
                                let shared_state = &lvl2.vm_table[lvl2_index];
                                match shared_state {
                                    SharedState::NotPresent => Err(MemoryError::NotFound),
                                    SharedState::OwnedTable(rw_lock)
                                    | SharedState::RefTable(rw_lock) => {
                                        let lvl1 = rw_lock.read();
                                        let shared_state = lvl1.phys_table.get(lvl1_index);

                                        if shared_state.is_present_set() {
                                            Ok(shared_state.get_phy_address())
                                        } else {
                                            Err(MemoryError::NotFound)
                                        }
                                    }
                                    SharedState::OwnedEntry | SharedState::RefEntry => Ok(lvl2
                                        .phys_table
                                        .get(lvl2_index)
                                        .get_next_entry_phy_address()
                                        | (vaddr & (PageMapLvl2::SIZE_PER_INDEX - 1))),
                                }
                            }
                            SharedState::OwnedEntry | SharedState::RefEntry => {
                                Ok(lvl3.phys_table.get(lvl3_index).get_next_entry_phy_address()
                                    | (vaddr & (PageMapLvl3::SIZE_PER_INDEX - 1)))
                            }
                        }
                    }
                    _ => Err(MemoryError::TableNotSupported),
                }
            }
            _ => Err(MemoryError::TableNotSupported),
        }
    }

    pub unsafe fn load(&self) -> Result<(), MemoryError> {
        // Already loaded, don't need to do anything!
        if self.is_loaded() {
            return Ok(());
        }

        match self.state.read().clone() {
            SharedState::RefTable(our_table) | SharedState::OwnedTable(our_table) => {
                let mut global_table = CURRENTLY_LOADED_TABLE.state.write();

                let table_vptr = our_table.read().phys_table.table_ptr();
                let phys_ptr = virt_to_phys(table_vptr).ok_or(MemoryError::InvalidPageTable)?;

                *global_table = SharedState::RefTable(our_table);
                unsafe { arch::registers::cr3::set_page_directory_base_register(phys_ptr) };

                Ok(())
            }
            SharedState::NotPresent => Err(MemoryError::InvalidPageTable),
            _ => panic!("Table does not support 'Entry' types, yet one was present!"),
        }
    }

    pub fn upgrade_mut<F, R>(&self, func: F) -> R
    where
        F: FnOnce(&mut SharedLvl4) -> R,
    {
        let mut inner = self.state.write();

        match &*inner {
            SharedState::NotPresent => {
                // Needs to be put into its box, because tables cannot be moved!
                let new_entry = Arc::new(RwLock::new(SharedLvl4::new()));
                *inner = SharedState::OwnedTable(new_entry.clone());

                let mut writeable = new_entry.write();
                func(&mut *writeable)
            }
            SharedState::OwnedTable(rw_lock) => {
                let mut writeable = rw_lock.write();
                func(&mut *writeable)
            }
            SharedState::RefTable(rw_lock) => {
                let table = rw_lock.clone();
                let inner_table = table.read();

                let writeable = Arc::new(RwLock::new(SharedLvl4 {
                    phys_table: inner_table.phys_table.clone(),
                    vm_table: inner_table.vm_table.clone(),
                }));

                let ret = func(&mut *writeable.write());
                *inner = SharedState::OwnedTable(writeable);

                ret
            }
            SharedState::OwnedEntry | SharedState::RefEntry => panic!(
                "Table does not support Large Page entries, yet one was found! Assuming invalid state!"
            ),
        }
    }

    pub unsafe fn new_from_bootloader() -> Self {
        let new_table = Self::empty();
        let cr3_lvl4 =
            arch::registers::cr3::get_page_directory_base_register() as *const PageMapLvl4;

        fn bl1(bl_lvl1: &PageMapLvl1, lvl1_shared: &mut SharedLvl1) {
            bl_lvl1
                .entry_iter()
                .enumerate()
                .for_each(|(lvl1_index, lvl1_entry)| {
                    lvl1_shared.phys_table.store(lvl1_entry, lvl1_index);
                });
        }

        fn bl2(bl_lvl2: &PageMapLvl2, lvl2_shared: &mut SharedLvl2) {
            bl_lvl2
                .entry_iter()
                .enumerate()
                .for_each(|(lvl2_index, bl_entry_lvl2)| {
                    if let Some(bl_lvl1) = unsafe { bl_entry_lvl2.get_table() } {
                        lvl2_shared.upgrade_mut(lvl2_index, |lvl1_shared, lvl2_entry| {
                            *lvl2_entry = bl_entry_lvl2;

                            bl1(bl_lvl1, lvl1_shared);
                        })
                    } else if bl_entry_lvl2.is_present_set() {
                        lvl2_shared
                            .entry_mut(lvl2_index, || {
                                PageEntry2M::convert_entry(bl_entry_lvl2)
                                    .ok_or(MemoryError::InvalidPageTable)
                            })
                            .unwrap();
                    }
                });
        }

        fn bl3(bl_lvl3: &PageMapLvl3, lvl3_shared: &mut SharedLvl3) {
            bl_lvl3
                .entry_iter()
                .enumerate()
                .for_each(|(lvl3_index, bl_entry_lvl3)| {
                    if let Some(bl_lvl2) = unsafe { bl_entry_lvl3.get_table() } {
                        lvl3_shared.upgrade_mut(lvl3_index, |lvl2_shared, lvl3_entry| {
                            *lvl3_entry = bl_entry_lvl3;

                            bl2(bl_lvl2, lvl2_shared);
                        });
                    } else if bl_entry_lvl3.is_present_set() {
                        lvl3_shared
                            .entry_mut(lvl3_index, || {
                                PageEntry1G::convert_entry(bl_entry_lvl3)
                                    .ok_or(MemoryError::InvalidPageTable)
                            })
                            .unwrap();
                    }
                });
        }

        new_table.upgrade_mut(|shared_lvl4| {
            let bl_lvl4 = unsafe { &*cr3_lvl4 };
            bl_lvl4
                .entry_iter()
                .enumerate()
                .for_each(|(lvl4_index, bl_entry_lvl4)| {
                    let Some(bl_lvl3) = (unsafe { bl_entry_lvl4.get_table() }) else {
                        return ();
                    };

                    shared_lvl4.upgrade_mut(lvl4_index, |lvl3_shared, lvl4_entry| {
                        *lvl4_entry = bl_entry_lvl4;

                        bl3(bl_lvl3, lvl3_shared);
                    });
                });
        });

        new_table
    }

    pub fn map_4k_page(
        &self,
        vpage: VirtPage,
        ppage: PhysPage,
        permissions: VmPermissions,
    ) -> Result<(), MemoryError> {
        let (lvl4_index, lvl3_index, lvl2_index, _) =
            table_indexes_for(vpage.0 as u64 * PAGE_4K as u64);

        self.upgrade_mut(|lvl4| {
            lvl4.upgrade_mut(lvl4_index, |lvl3, lvl4_entry| {
                lvl4_entry.add_permissions_from(permissions);

                lvl3.upgrade_mut(lvl3_index, |lvl2, lvl3_entry| {
                    lvl3_entry.add_permissions_from(permissions);

                    lvl2.upgrade_mut(lvl2_index, |lvl1, lvl2_entry| {
                        lvl2_entry.add_permissions_from(permissions);

                        lvl1.link_page(vpage, ppage, permissions);
                    })
                })
            })
        });

        Ok(())
    }

    pub fn _map_2m_page(
        &self,
        vpage: VirtPage,
        ppage: PhysPage,
        permissions: VmPermissions,
    ) -> Result<(), MemoryError> {
        if (vpage.0 * PAGE_4K) & (PAGE_2M - 1) != 0 {
            return Err(MemoryError::NotPageAligned);
        }

        let (lvl4_index, lvl3_index, lvl2_index, _) =
            table_indexes_for(vpage.0 as u64 * PAGE_4K as u64);

        self.upgrade_mut(|lvl4| {
            lvl4.upgrade_mut(lvl4_index, |lvl3, lvl4_entry| {
                lvl4_entry.set_present_flag(permissions.is_read_set());
                lvl4_entry.set_read_write_flag(permissions.is_write_set());
                lvl4_entry.set_user_access_flag(permissions.is_user_set());
                lvl4_entry.set_execute_disable_flag(!permissions.is_exec_set());

                lvl3.upgrade_mut(lvl3_index, |lvl2, lvl3_entry| {
                    lvl3_entry.set_present_flag(permissions.is_read_set());
                    lvl3_entry.set_read_write_flag(permissions.is_write_set());
                    lvl3_entry.set_user_access_flag(permissions.is_user_set());
                    lvl3_entry.set_execute_disable_flag(!permissions.is_exec_set());

                    lvl2.entry_mut(lvl2_index, || {
                        let mut entry = PageEntry2M::new();

                        entry.set_present_flag(permissions.is_read_set());
                        entry.set_read_write_flag(permissions.is_write_set());
                        entry.set_user_access_flag(permissions.is_user_set());
                        entry.set_execute_disable_flag(!permissions.is_exec_set());

                        entry.set_phy_address(ppage.0 * PAGE_4K as u64);

                        Ok(entry)
                    })
                })
            })
        })?;

        Ok(())
    }

    pub fn _map_1g_page(
        &self,
        vpage: VirtPage,
        ppage: PhysPage,
        permissions: VmPermissions,
    ) -> Result<(), MemoryError> {
        if (vpage.0 * PAGE_4K) & (PAGE_1G - 1) != 0 {
            return Err(MemoryError::NotPageAligned);
        }

        let (lvl4_index, lvl3_index, _, _) = table_indexes_for(vpage.0 as u64 * PAGE_4K as u64);

        self.upgrade_mut(|lvl4| {
            lvl4.upgrade_mut(lvl4_index, |lvl3, lvl4_entry| {
                lvl4_entry.set_present_flag(permissions.is_read_set());
                lvl4_entry.set_read_write_flag(permissions.is_write_set());
                lvl4_entry.set_user_access_flag(permissions.is_user_set());
                lvl4_entry.set_execute_disable_flag(!permissions.is_exec_set());

                lvl3.entry_mut(lvl3_index, || {
                    let mut entry = PageEntry1G::new();

                    entry.set_present_flag(permissions.is_read_set());
                    entry.set_read_write_flag(permissions.is_write_set());
                    entry.set_user_access_flag(permissions.is_user_set());
                    entry.set_execute_disable_flag(!permissions.is_exec_set());

                    entry.set_phy_address(ppage.0 * PAGE_4K as u64);

                    Ok(entry)
                })
            })
        })?;

        Ok(())
    }
}

impl Clone for SharedTable {
    fn clone(&self) -> Self {
        let mut new_table = Self::empty();
        new_table.state = RwLock::new(self.state.read().clone());

        new_table
    }
}

impl<T> Clone for SharedState<T> {
    fn clone(&self) -> Self {
        match self {
            SharedState::NotPresent => SharedState::NotPresent,
            SharedState::OwnedTable(rw_lock) | SharedState::RefTable(rw_lock) => {
                SharedState::RefTable(rw_lock.clone())
            }
            SharedState::OwnedEntry | SharedState::RefEntry => SharedState::RefEntry,
        }
    }
}

impl SharedLvl4 {
    pub const fn new() -> Self {
        Self {
            phys_table: PageMapLvl4::new(),
            vm_table: [const { SharedState::NotPresent }; 512],
        }
    }

    /// Gets a refrence to the inner structure, but does not upgrade it.
    pub fn _inner_ref<F, R>(&self, index: usize, func: F) -> R
    where
        F: FnOnce(Option<&SharedLvl3>, &PageEntryLvl4) -> R,
    {
        if let Some(locked) = self.vm_table.get(index).and_then(|inner| match inner {
            SharedState::OwnedTable(rw_lock) | SharedState::RefTable(rw_lock) => {
                Some(rw_lock.read())
            }
            _ => None,
        }) {
            func(Some(&*locked), &self.phys_table.get(index))
        } else {
            func(None, &self.phys_table.get(index))
        }
    }

    /// Convert/Upgrade this index to a `OwnTable` and update `phys_table`.
    ///
    /// This function will write the physical address of the entry into the
    /// `PageEntry` for you.
    pub fn upgrade_mut<F, R>(&mut self, index: usize, func: F) -> R
    where
        F: FnOnce(&mut SharedLvl3, &mut PageEntryLvl4) -> R,
    {
        let mut entry = self.phys_table.get(index);

        let ret = match self.vm_table[index] {
            SharedState::NotPresent => {
                // Needs to be put into its box, because tables cannot be moved!
                let new_entry = Arc::new(RwLock::new(SharedLvl3::new()));
                self.vm_table[index] = SharedState::OwnedTable(new_entry.clone());

                let mut writeable = new_entry.write();
                let ret = func(&mut *writeable, &mut entry);
                entry.set_next_entry_phy_address(
                    virt_to_phys(writeable.phys_table.table_ptr())
                        .expect("Cannot locate the physical ptr of page table!"),
                );

                ret
            }
            SharedState::OwnedTable(ref rw_lock) => {
                let mut writeable = rw_lock.write();
                let ret = func(&mut *writeable, &mut entry);
                entry.set_next_entry_phy_address(
                    virt_to_phys(writeable.phys_table.table_ptr())
                        .expect("Cannot locate the physical ptr of page table!"),
                );

                ret
            }
            SharedState::RefTable(ref rw_lock) => {
                let table = rw_lock.clone();
                let inner_table = table.read();

                let writeable = Arc::new(RwLock::new(SharedLvl3 {
                    phys_table: inner_table.phys_table.clone(),
                    vm_table: inner_table.vm_table.clone(),
                }));

                let ret = func(&mut *writeable.write(), &mut entry);
                entry.set_next_entry_phy_address(
                    virt_to_phys(writeable.read().phys_table.table_ptr())
                        .expect("Cannot locate the physical ptr of page table!"),
                );

                self.vm_table[index] = SharedState::OwnedTable(writeable);

                ret
            }
            SharedState::OwnedEntry | SharedState::RefEntry => panic!(
                "Table does not support Large Page entries, yet one was found! Assuming invalid state!"
            ),
        };

        assert!(entry.is_present_set());
        assert!(!entry.is_page_size_set());

        self.phys_table.store(entry, index);

        ret
    }
}

impl SharedLvl3 {
    pub const fn new() -> Self {
        Self {
            phys_table: PageMapLvl3::new(),
            vm_table: [const { SharedState::NotPresent }; 512],
        }
    }

    fn upgrade_entry(&mut self, entry: usize) {
        let state = &mut self.vm_table[entry];

        match state {
            SharedState::NotPresent => {
                // Make a new table
                *state = SharedState::OwnedTable(Arc::new(RwLock::new(SharedLvl2::new())));
            }
            SharedState::RefTable(rw_lock) => {
                let table = rw_lock.clone();
                let inner_table = table.read();

                *state = SharedState::OwnedTable(Arc::new(RwLock::new(SharedLvl2 {
                    phys_table: inner_table.phys_table.clone(),
                    vm_table: inner_table.vm_table.clone(),
                })));
            }
            _ => (),
        }
    }

    /// Gets a refrence to the inner structure, but does not upgrade it.
    pub fn _inner_ref<F, R>(&self, index: usize, func: F) -> R
    where
        F: FnOnce(Option<&SharedLvl2>, &PageEntryLvl3) -> R,
    {
        if let Some(locked) = self.vm_table.get(index).and_then(|inner| match inner {
            SharedState::OwnedTable(rw_lock) | SharedState::RefTable(rw_lock) => {
                Some(rw_lock.read())
            }
            _ => None,
        }) {
            func(Some(&*locked), &self.phys_table.get(index))
        } else {
            func(None, &self.phys_table.get(index))
        }
    }

    /// Upgrade this index to a 'OwnedEntry' and update the `phys_table` with PageEntry1G
    pub fn entry_mut<F, E>(&mut self, index: usize, func: F) -> Result<(), E>
    where
        F: FnOnce() -> Result<PageEntry1G, E>,
    {
        let ret = func()?;

        assert!(
            ret.is_page_size_set(),
            "Large entry must have 'page_size' flag set!"
        );

        self.vm_table[index] = SharedState::OwnedEntry;
        self.phys_table.store(ret, index);

        Ok(())
    }

    /// Convert/Upgrade this index to a `OwnTable` and update `phys_table`.
    ///
    /// This function will write the physical address of the entry into the
    /// `PageEntry` for you.
    pub fn upgrade_mut<F, R>(&mut self, index: usize, func: F) -> R
    where
        F: FnOnce(&mut SharedLvl2, &mut PageEntryLvl3) -> R,
    {
        let mut entry = self.phys_table.get(index);

        let ret = match self.vm_table[index] {
            SharedState::OwnedEntry | SharedState::RefEntry | SharedState::NotPresent => {
                entry = PageEntryLvl3::zero();
                // Needs to be put into its box, because tables cannot be moved!
                let new_entry = Arc::new(RwLock::new(SharedLvl2::new()));
                self.vm_table[index] = SharedState::OwnedTable(new_entry.clone());

                let mut writeable = new_entry.write();
                let ret = func(&mut *writeable, &mut entry);
                entry.set_next_entry_phy_address(
                    virt_to_phys(writeable.phys_table.table_ptr())
                        .expect("Cannot locate the physical ptr of page table!"),
                );

                ret
            }
            SharedState::OwnedTable(ref rw_lock) => {
                let mut writeable = rw_lock.write();
                let ret = func(&mut *writeable, &mut entry);
                entry.set_next_entry_phy_address(
                    virt_to_phys(writeable.phys_table.table_ptr())
                        .expect("Cannot locate the physical ptr of page table!"),
                );

                ret
            }
            SharedState::RefTable(_) => {
                self.upgrade_entry(index);

                let SharedState::OwnedTable(ref table) = self.vm_table[index] else {
                    panic!("Upgraded table, but table is not owned!");
                };

                let mut writeable = table.write();
                let ret = func(&mut *writeable, &mut entry);
                entry.set_next_entry_phy_address(
                    virt_to_phys(writeable.phys_table.table_ptr())
                        .expect("Cannot locate the physical ptr of page table!"),
                );

                ret
            }
        };

        assert!(entry.is_present_set());
        assert!(!entry.is_page_size_set());

        self.phys_table.store(entry, index);

        ret
    }
}

impl SharedLvl2 {
    pub const fn new() -> Self {
        Self {
            phys_table: PageMapLvl2::new(),
            vm_table: [const { SharedState::NotPresent }; 512],
        }
    }

    fn upgrade_entry(&mut self, entry: usize) {
        let state = &mut self.vm_table[entry];

        match state {
            SharedState::NotPresent => {
                // Make a new table
                *state = SharedState::OwnedTable(Arc::new(RwLock::new(SharedLvl1::new())));
            }
            SharedState::RefTable(rw_lock) => {
                let table = rw_lock.clone();
                let inner_table = table.read();

                *state = SharedState::OwnedTable(Arc::new(RwLock::new(SharedLvl1 {
                    phys_table: inner_table.phys_table.clone(),
                })));
            }
            _ => (),
        }
    }

    /// Gets a refrence to the inner structure, but does not upgrade it.
    pub fn _inner_ref<F, R>(&self, index: usize, func: F) -> R
    where
        F: FnOnce(Option<&SharedLvl1>, &PageEntryLvl2) -> R,
    {
        if let Some(locked) = self.vm_table.get(index).and_then(|inner| match inner {
            SharedState::OwnedTable(rw_lock) | SharedState::RefTable(rw_lock) => {
                Some(rw_lock.read())
            }
            _ => None,
        }) {
            func(Some(&*locked), &self.phys_table.get(index))
        } else {
            func(None, &self.phys_table.get(index))
        }
    }
    ///
    /// Upgrade this index to a 'OwnedEntry' and update the `phys_table` with PageEntry1G
    pub fn entry_mut<F, E>(&mut self, index: usize, func: F) -> Result<(), E>
    where
        F: FnOnce() -> Result<PageEntry2M, E>,
    {
        let ret = func()?;

        assert!(
            ret.is_page_size_set(),
            "Large entry must have 'page_size' flag set!"
        );

        self.vm_table[index] = SharedState::OwnedEntry;
        self.phys_table.store(ret, index);

        Ok(())
    }

    /// Convert/Upgrade this index to a `OwnTable` and update `phys_table`.
    ///
    /// This function will write the physical address of the entry into the
    /// `PageEntry` for you.
    pub fn upgrade_mut<F, R>(&mut self, index: usize, func: F) -> R
    where
        F: FnOnce(&mut SharedLvl1, &mut PageEntryLvl2) -> R,
    {
        let mut entry = self.phys_table.get(index);

        let ret = match self.vm_table[index] {
            SharedState::OwnedEntry | SharedState::RefEntry | SharedState::NotPresent => {
                entry = PageEntryLvl2::zero();

                // Needs to be put into its box, because tables cannot be moved!
                let new_entry = Arc::new(RwLock::new(SharedLvl1::new()));
                self.vm_table[index] = SharedState::OwnedTable(new_entry.clone());

                let mut writeable = new_entry.write();
                let ret = func(&mut *writeable, &mut entry);
                entry.set_next_entry_phy_address(
                    virt_to_phys(writeable.phys_table.table_ptr())
                        .expect("Cannot locate the physical ptr of page table!"),
                );

                ret
            }
            SharedState::OwnedTable(ref rw_lock) => {
                let mut writeable = rw_lock.write();
                let ret = func(&mut *writeable, &mut entry);
                entry.set_next_entry_phy_address(
                    virt_to_phys(writeable.phys_table.table_ptr())
                        .expect("Cannot locate the physical ptr of page table!"),
                );

                ret
            }
            SharedState::RefTable(_) => {
                self.upgrade_entry(index);

                let SharedState::OwnedTable(ref table) = self.vm_table[index] else {
                    panic!("Upgraded table, but table is not owned!");
                };

                let mut writeable = table.write();
                let ret = func(&mut *writeable, &mut entry);
                entry.set_next_entry_phy_address(
                    virt_to_phys(writeable.phys_table.table_ptr())
                        .expect("Cannot locate the physical ptr of page table!"),
                );

                ret
            }
        };

        assert!(entry.is_present_set());
        assert!(!entry.is_page_size_set());

        self.phys_table.store(entry, index);

        ret
    }
}

impl SharedLvl1 {
    pub const fn new() -> Self {
        Self {
            phys_table: PageMapLvl1::new(),
        }
    }

    /// This function calculates the realative offset of vpage in its **OWN** table.
    ///
    /// The caller still needs to ensure the page table entries that point to `Self`
    /// are correctly aligned to point to `vpage`!
    fn link_page(&mut self, vpage: VirtPage, ppage: PhysPage, permissions: VmPermissions) {
        let mut page_entry = PageEntry4K::new();

        page_entry.add_permissions_from(permissions);
        page_entry.set_phy_address(ppage.0 * PAGE_4K as u64);

        let table_index = vpage.0 % 512;
        self.phys_table.store(page_entry, table_index);
    }
}

impl Display for SharedTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let state = self.state.read();
        write!(f, "SharedTable ")?;

        match &*state {
            SharedState::NotPresent => writeln!(f, "(NotPresent)")?,
            SharedState::OwnedTable(rw_lock) => {
                writeln!(f, "(Owned) {}", rw_lock.read())?;
            }
            SharedState::RefTable(rw_lock) => {
                writeln!(f, "(Ref) {}", rw_lock.read())?;
            }
            SharedState::OwnedEntry | SharedState::RefEntry => writeln!(f, "(Invalid)")?,
        }

        Ok(())
    }
}

impl Display for SharedLvl4 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "SharedLvl4")?;

        for (index, entry) in self.vm_table.iter().enumerate() {
            let phys_entry = self.phys_table.get(index);

            match entry {
                SharedState::OwnedTable(rw_lock) => {
                    writeln!(f, " |-{index:>3} {} (Owned) {}", phys_entry, rw_lock.read())?;
                }
                SharedState::RefTable(rw_lock) => {
                    writeln!(f, " |-{index:>3} {} (Ref) {}", phys_entry, rw_lock.read())?;
                }
                _ => (),
            }
        }

        Ok(())
    }
}

impl Display for SharedLvl3 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "SharedLvl3")?;

        for (index, entry) in self.vm_table.iter().enumerate() {
            let phys_entry = self.phys_table.get(index);

            match entry {
                SharedState::OwnedTable(rw_lock) => {
                    writeln!(
                        f,
                        " |  |-{index:>3} {} (Owned) {}",
                        phys_entry,
                        rw_lock.read()
                    )?;
                }
                SharedState::RefTable(rw_lock) => {
                    writeln!(
                        f,
                        " |  |-{index:>3} {} (Ref) {}",
                        phys_entry,
                        rw_lock.read()
                    )?;
                }
                SharedState::OwnedEntry => writeln!(
                    f,
                    " |  |-{index:^3} {} (Owned) [V{:16x} -> P{:16x}]",
                    phys_entry,
                    index as u64 * PageMapLvl3::SIZE_PER_INDEX,
                    PageEntry1G::convert_entry(phys_entry)
                        .unwrap()
                        .get_phy_address()
                )?,
                SharedState::RefEntry => writeln!(
                    f,
                    " |  |-{index:^3} {} (Ref) [V{:016x}] -> P{:16x}]",
                    phys_entry,
                    index as u64 * PageMapLvl3::SIZE_PER_INDEX,
                    PageEntry1G::convert_entry(phys_entry)
                        .unwrap()
                        .get_phy_address()
                )?,
                _ => (),
            }
        }

        Ok(())
    }
}

impl Display for SharedLvl2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "SharedLvl2")?;

        for (index, entry) in self.vm_table.iter().enumerate() {
            let phys_entry = self.phys_table.get(index);

            match entry {
                SharedState::OwnedTable(rw_lock) => {
                    writeln!(
                        f,
                        " |  |  |-{index:>3} {} (Owned) {}",
                        phys_entry,
                        rw_lock.read()
                    )?;
                }
                SharedState::RefTable(rw_lock) => {
                    writeln!(
                        f,
                        " |  |  |-{index:>3} {} (Ref) {}",
                        phys_entry,
                        rw_lock.read()
                    )?;
                }
                SharedState::OwnedEntry => writeln!(
                    f,
                    " |  |  |-{index:^3} {} (Owned) [V{:16x} -> P{:16x}]",
                    phys_entry,
                    index as u64 * PageMapLvl2::SIZE_PER_INDEX,
                    PageEntry2M::convert_entry(phys_entry)
                        .unwrap()
                        .get_phy_address()
                )?,
                SharedState::RefEntry => writeln!(
                    f,
                    " |  |  |-{index:^3} {} (Ref) [V{:016x}] -> P{:16x}]",
                    phys_entry,
                    index as u64 * PageMapLvl2::SIZE_PER_INDEX,
                    PageEntry2M::convert_entry(phys_entry)
                        .unwrap()
                        .get_phy_address()
                )?,
                _ => (),
            }
        }

        Ok(())
    }
}

impl Display for SharedLvl1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "SharedLvl1")?;

        for (index, entry) in self.phys_table.entry_iter().enumerate() {
            if entry.is_present_set() {
                writeln!(
                    f,
                    " |  |  |  |-{index:^3} {} [V{:016x}] -> P{:16x}]",
                    entry,
                    index as u64 * PageMapLvl1::SIZE_PER_INDEX,
                    entry.get_phy_address()
                )?;
            }
        }

        Ok(())
    }
}

pub trait PagePermissions {
    fn reduce_permissions_to(&mut self, permissions: VmPermissions);
    fn add_permissions_from(&mut self, permissions: VmPermissions);
}

macro_rules! page_permissions_for {
    ($($ty:ty),*) => {
        $(
            impl PagePermissions for $ty {
                fn reduce_permissions_to(&mut self, permissions: VmPermissions) {
                    self.set_present_flag(permissions.is_read_set());
                    self.set_read_write_flag(permissions.is_write_set());
                    self.set_user_access_flag(permissions.is_user_set());
                    self.set_execute_disable_flag(!permissions.is_exec_set());
                }

                fn add_permissions_from(&mut self, permissions: VmPermissions) {
                    self.set_present_flag(permissions.is_read_set() | self.is_present_set());
                    self.set_read_write_flag(permissions.is_write_set() | self.is_read_write_set());
                    self.set_user_access_flag(permissions.is_user_set() | self.is_user_access_set());
                    self.set_execute_disable_flag(
                        !(permissions.is_exec_set() | !self.is_execute_disable_set()),
                    );
                }
            }
        )*
    };
}

page_permissions_for! { PageEntry1G, PageEntry2M, PageEntry4K, PageEntryLvl2, PageEntryLvl3, PageEntryLvl4 }

impl Debug for SharedTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SharedTable").finish_non_exhaustive()
    }
}
