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
use core::ops::{Add, AddAssign, BitOr, BitOrAssign};

use crate::{
    addr::{PhysAddr, VirtAddr},
    virt2phys::PhysPtrTranslationError,
};
use crate::{
    page::{PhysPage, VirtPage},
    virt2phys::ObtainPhysAddr,
};
use alloc::{boxed::Box, sync::Arc};
use arch::{
    paging64::{
        PageEntry1G, PageEntry2M, PageEntry4K, PageEntryLvl2, PageEntryLvl3, PageEntryLvl4,
        PageMapLvl1, PageMapLvl2, PageMapLvl3, PageMapLvl4,
    },
    registers::cr3,
};
use hw::make_hw;
use lldebug::logln;
use spin::RwLock;
use util::consts::PAGE_4K;

/// The top-most page table
pub struct Virt2PhysMapping {
    mapping: RwLock<Option<Arc<RwLock<SafePageMapLvl4>>>>,
}

#[derive(Clone)]
struct SafePageMapLvl4 {
    table: PageMapLvl4,
    lower: [Option<Box<SafePageMapLvl3>>; 512],
}

#[derive(Clone)]
struct SafePageMapLvl3 {
    table: PageMapLvl3,
    lower: [Option<Box<SafePageMapLvl2>>; 512],
}

#[derive(Clone)]
struct SafePageMapLvl2 {
    table: PageMapLvl2,
    lower: [Option<Box<SafePageMapLvl1>>; 512],
}

#[derive(Clone)]
struct SafePageMapLvl1 {
    table: PageMapLvl1,
}

impl Clone for Virt2PhysMapping {
    fn clone(&self) -> Self {
        Virt2PhysMapping {
            mapping: RwLock::new(self.mapping.read().clone()),
        }
    }
}

/// Flush this page from the TLB
#[inline]
pub unsafe fn flush_tlb(page: VirtPage) {
    unsafe {
        core::arch::asm!(
            "invlpg [rax]",
            in("rax") page.addr().addr(),
            options(nostack)
        )
    }
}

/// Returns the indexes into the page tables that would give this virtual address.
const fn table_indexes_for(vaddr: VirtAddr) -> (usize, usize, usize, usize) {
    let lvl4 = PageMapLvl4::addr2index(vaddr.addr() as u64).unwrap();
    let lvl3 = PageMapLvl3::addr2index(vaddr.addr() as u64 % PageMapLvl4::SIZE_PER_INDEX).unwrap();
    let lvl2 = PageMapLvl2::addr2index(vaddr.addr() as u64 % PageMapLvl3::SIZE_PER_INDEX).unwrap();
    let lvl1 = PageMapLvl1::addr2index(vaddr.addr() as u64 % PageMapLvl2::SIZE_PER_INDEX).unwrap();

    (lvl4, lvl3, lvl2, lvl1)
}

static CURRENTLY_LOADED_PAGE_TABLES: Virt2PhysMapping = Virt2PhysMapping::empty();

/// Bootloader convert a virtual address to a physical address
pub fn bootloader_convert_phys(virt: u64) -> Option<u64> {
    // FIXME: This is just hardcoded for now, but should be populated from the bootloader!!!!
    //
    // Since we know the bootloader is identity mapped, the physical PTRs are valid virtual PTRs!
    let (lvl4_idx, lvl3_idx, lvl2_idx, lvl1_idx) = table_indexes_for(VirtAddr::new(virt as usize));

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

/// Convert a virtual address to a physical address
pub fn virt_to_phys(virt: VirtAddr) -> Result<PhysAddr, PhysPtrTranslationError> {
    match CURRENTLY_LOADED_PAGE_TABLES.vpage_to_ppage_lookup(VirtPage::containing_addr(virt)) {
        // Try to lookup in the page tables loaded by us
        Ok(phys_page) => Ok(phys_page.addr().extend_by(virt.chop_bottom(PAGE_4K))),
        // If we haven't loaded the page tables yet (maybe in progress of loading them) we
        // try lookin up the addr in the old bootloader page tables.
        Err(PhysPtrTranslationError::PageEntriesNotSetup) => {
            bootloader_convert_phys(virt.addr() as u64)
                .inspect(|inner| {
                    logln!("Virt2Phys (Bootloader) - V{:#016x} P{:#016x}", virt, inner)
                })
                .ok_or(PhysPtrTranslationError::VirtNotFound(virt))
                .map(|phys_addr| PhysAddr::new(phys_addr as usize))
        }
        // Our loaded page tables gave a non "I am not loaded yet"-kinda error, so
        // lets just pass it along.
        Err(e) => Err(e),
    }
}

/// Hook the `virt_to_phys` function to the `phys_page` trait provider
pub fn init_virt2phys_provider() {
    crate::virt2phys::set_global_lookup_fn(virt_to_phys);
}

#[derive(Clone, Copy, Debug)]
pub enum PageTableLoadingError {
    PhysTranslationErr(PhysPtrTranslationError),
    AlreadyLoaded,
    Empty,
}

impl Virt2PhysMapping {
    /// Create a new table that is not mapped
    pub const fn empty() -> Self {
        Self {
            mapping: RwLock::new(None),
        }
    }

    /// Copy all entries and tables from 'from' into a new table.
    pub fn inhearit_from(from: &Self) -> Self {
        let old_table = from
            .mapping
            .read()
            .as_ref()
            .map(|inner| Arc::new(RwLock::new(inner.read().clone())));

        Self {
            mapping: RwLock::new(old_table),
        }
    }

    /// Copy all entries from the bootloader's page tables
    pub unsafe fn inhearit_bootloader() -> Result<Self, PageCorrelationError> {
        let new_table = Self::empty();

        // Since the bootloader identity maps itself, we can use that to read the page tables
        // even though their PTRs are PhysAddr instead of VirtAddr.
        let cr3_lvl4 =
            arch::registers::cr3::get_page_directory_base_register() as *const PageMapLvl4;

        let bootloader_lvl4 = unsafe { &*cr3_lvl4 };

        // FIXME: Once we have Huge page table support, we should inhearit the whole thing,
        //        but for right now we should skip the bootloader's identity mapping and just
        //        try to map the kernel.
        const SKIP_UNTIL_VADDR: usize = 0x100000000000;

        // FIXME: We should protect the kernel's image once we get here instead of just
        //        spamming perms until it works :)
        const KERNEL_RWE: VmPermissions = VmPermissions::none()
            .set_exec_flag(true)
            .set_read_flag(true)
            .set_write_flag(true)
            .set_user_flag(false);

        // Since we know 'new_table' is not loaded, we will eventually have to write it
        // to cr3 so we dont need to invlpg.
        //
        // We are also making new page tables, so we shoulnt need any special options.
        const KERNEL_OPT: VmOptions = VmOptions::none().set_no_tlb_flush_flag(true);

        // Map a 1Gib page to our page table
        fn map_1g(
            mapping: &Virt2PhysMapping,
            vpage: VirtPage,
            ppage: PhysPage,
        ) -> Result<(), PageCorrelationError> {
            if vpage.addr().addr() < SKIP_UNTIL_VADDR {
                return Ok(());
            }

            for page in 0..512 {
                map_2m(
                    mapping,
                    vpage + VirtPage::new(page),
                    ppage + PhysPage::new(page),
                )?;
            }

            Ok(())
        }

        // Map a 2Mib page to our page table
        fn map_2m(
            mapping: &Virt2PhysMapping,
            vpage: VirtPage,
            ppage: PhysPage,
        ) -> Result<(), PageCorrelationError> {
            if vpage.addr().addr() < SKIP_UNTIL_VADDR {
                return Ok(());
            }

            for page in 0..512 {
                map_4k(
                    mapping,
                    vpage + VirtPage::new(page),
                    ppage + PhysPage::new(page),
                )?;
            }

            Ok(())
        }

        // Map a normal 4Kib page to our page table
        fn map_4k(
            mapping: &Virt2PhysMapping,
            vpage: VirtPage,
            ppage: PhysPage,
        ) -> Result<(), PageCorrelationError> {
            if vpage.addr().addr() < SKIP_UNTIL_VADDR {
                return Ok(());
            }

            mapping.correlate_page(vpage, ppage, KERNEL_OPT, KERNEL_RWE)?;
            Ok(())
        }

        for (lvl4_index, lvl4_entry) in bootloader_lvl4.entry_iter().enumerate() {
            // Attempt to get the next (Lvl4) table entries
            let Some(lvl3) = (unsafe { lvl4_entry.get_table() }) else {
                continue;
            };

            for (lvl3_index, lvl3_entry) in lvl3.entry_iter().enumerate() {
                // Attempt to get the next (Lvl3) table entries
                let Some(lvl2) = (unsafe { lvl3_entry.get_table() }) else {
                    // 1Gib Entry
                    if lvl3_entry.is_present_set() && lvl3_entry.is_page_size_set() {
                        map_1g(
                            &new_table,
                            // Convert the indexes back into a page id
                            VirtPage::new(lvl4_index * 512 * 512 * 512 + lvl3_index * 512 * 512),
                            PhysPage::containing_addr(PhysAddr::new(
                                PageEntry1G::convert_entry(lvl3_entry)
                                    .expect("Expected 1Gib entry")
                                    .get_phy_address() as usize,
                            )),
                        )?;
                    }
                    continue;
                };

                for (lvl2_index, lvl2_entry) in lvl2.entry_iter().enumerate() {
                    // Attempt to get the next (Lvl2) table entries
                    let Some(lvl1) = (unsafe { lvl2_entry.get_table() }) else {
                        // 2Mib Entry
                        if lvl2_entry.is_present_set() && lvl2_entry.is_page_size_set() {
                            map_2m(
                                &new_table,
                                // Convert the indexes back into a page id
                                VirtPage::new(
                                    lvl4_index * 512 * 512 * 512
                                        + lvl3_index * 512 * 512
                                        + lvl2_index * 512,
                                ),
                                PhysPage::containing_addr(PhysAddr::new(
                                    PageEntry2M::convert_entry(lvl2_entry)
                                        .expect("Expected 2Mib entry")
                                        .get_phy_address()
                                        as usize,
                                )),
                            )?;
                        }
                        continue;
                    };

                    for (lvl1_index, lvl1_entry) in lvl1.entry_iter().enumerate() {
                        if !lvl1_entry.is_present_set() {
                            continue;
                        }

                        map_4k(
                            &new_table,
                            // Convert the indexes back into a page id
                            VirtPage::new(
                                lvl4_index * 512 * 512 * 512
                                    + lvl3_index * 512 * 512
                                    + lvl2_index * 512
                                    + lvl1_index,
                            ),
                            PhysPage::containing_addr(PhysAddr::new(
                                lvl1_entry.get_phy_address() as usize
                            )),
                        )?;
                    }
                }
            }
        }

        Ok(new_table)
    }

    /// Get the physical address of the inner table
    fn inner_table_ptr(&self) -> Result<PhysAddr, PageTableLoadingError> {
        let mapping_inner = self.mapping.read();
        let raw_ptr = VirtAddr::new(
            mapping_inner
                .as_ref()
                .ok_or(PageTableLoadingError::Empty)?
                .read()
                .table
                .table_ptr() as usize,
        );

        raw_ptr
            .phys_addr()
            .map_err(|phys_err| PageTableLoadingError::PhysTranslationErr(phys_err))
    }

    /// Check if this table is currently loaded
    pub fn is_loaded(&self) -> bool {
        // FIXME: I think this is a bug with RwLock, since there should only be
        //        read locks (I even checked), but it always hangs?
        unsafe { &*self.mapping.as_mut_ptr() }
            .as_ref()
            .is_some_and(|our_table| {
                unsafe { &*CURRENTLY_LOADED_PAGE_TABLES.mapping.as_mut_ptr() }
                    .as_ref()
                    .is_some_and(|global_table| Arc::ptr_eq(our_table, global_table))
            })
    }

    /// Load this table
    pub unsafe fn load(self) -> Result<(), PageTableLoadingError> {
        // We want to hold a lock to the global table to ensure no one else can change it
        let lock = CURRENTLY_LOADED_PAGE_TABLES.mapping.upgradeable_read();

        if self.is_loaded() {
            return Err(PageTableLoadingError::AlreadyLoaded);
        }

        let inner_table_ptr = self.inner_table_ptr()?;

        // Write ourselves to the table
        {
            let mut global_table = lock.upgrade();
            *global_table = self.mapping.read().clone();
        }

        // Write our addr to the page dir
        unsafe { cr3::set_page_directory_base_register(inner_table_ptr.addr() as u64) };

        Ok(())
    }

    /// Find the physical-page of a virtual-page by looking it up in the page tables
    pub fn vpage_to_ppage_lookup(
        &self,
        page: VirtPage,
    ) -> Result<PhysPage, PhysPtrTranslationError> {
        // FIXME: I think this is a bug with RwLock, since there should only be
        //        read locks (I even checked), but it always hangs?
        let mapping_lock = unsafe { &*self.mapping.as_mut_ptr() };
        let Some(inner) = mapping_lock.as_ref() else {
            return Err(PhysPtrTranslationError::PageEntriesNotSetup);
        };

        let (lvl4_index, lvl3_index, lvl2_index, lvl1_index) = table_indexes_for(page.addr());

        let Some(phys_addr) = inner.read().lower.get(lvl4_index).and_then(|lvl3| {
            let entry = lvl3
                .as_ref()?
                .lower
                .get(lvl3_index)?
                .as_ref()?
                .lower
                .get(lvl2_index)?
                .as_ref()?
                .table
                .get(lvl1_index);

            if entry.is_present_set() {
                Some(entry.get_phy_address())
            } else {
                None
            }
        }) else {
            return Err(PhysPtrTranslationError::VirtNotFound(page.addr()));
        };

        Ok(PhysPage::containing_addr(PhysAddr::new(phys_addr as usize)))
    }

    /// Map this `vpage` to `ppage` returning the previous PhysPage if there was one
    pub fn correlate_page(
        &self,
        vpage: VirtPage,
        ppage: PhysPage,
        options: VmOptions,
        permissions: VmPermissions,
    ) -> Result<Option<PhysPage>, PageCorrelationError> {
        // Grab a lock to this page table
        let lock = if options.is_dont_wait_for_lock_set() {
            match self.mapping.try_upgradeable_read() {
                Some(lock) => lock,
                None => return Err(PageCorrelationError::AlreadyLocked),
            }
        } else {
            self.mapping.upgradeable_read()
        };

        let (lvl4_index, lvl3_index, lvl2_index, lvl1_index) = table_indexes_for(vpage.addr());

        fn check_perms(
            present: bool,
            prev_perm: VmPermissions,
            new_perm: VmPermissions,
            options: VmOptions,
        ) -> Option<PageCorrelationError> {
            // If we are the ones making this table, there is no need to check permissions
            if !present {
                return None;
            }

            // Unless we are told to upgrade permissions, fail
            if prev_perm < new_perm && !options.is_increase_perm_set() {
                return Some(PageCorrelationError::ExistingPermissionsTooStrict {
                    table_perms: prev_perm,
                    requested_perms: new_perm,
                });
            }

            // Unless we are told to reduce permissions, fail
            if prev_perm > new_perm
                && !options.is_reduce_perm_from_tables_set()
                && options.is_overwrite_set()
            {
                return Some(PageCorrelationError::ExistingPermissionsPermissive {
                    table_perms: prev_perm,
                    requested_perms: new_perm,
                });
            }

            // If we pass all checks we can continue
            None
        }

        // We first try to do all the checks with RO before we commit to
        // fully locking the table.
        if let Some(err) = lock.as_ref().and_then(|ro_checks| {
            let lvl2 = |entry: &PageEntryLvl2, lower: &SafePageMapLvl1| {
                check_perms(
                    entry.is_present_set(),
                    entry.get_permissions(),
                    permissions,
                    options,
                )
                .or_else(|| {
                    let page_entry = lower.table.get(lvl1_index);

                    // Make sure we don't override unless we are told to
                    if page_entry.is_present_set() && !options.is_overwrite_set() {
                        return Some(PageCorrelationError::PageAlreadyMapped);
                    }

                    check_perms(
                        page_entry.is_present_set(),
                        page_entry.get_permissions(),
                        permissions,
                        options,
                    )
                })
            };
            let lvl3 = |entry: &PageEntryLvl3, lower: &SafePageMapLvl2| {
                check_perms(
                    entry.is_present_set(),
                    entry.get_permissions(),
                    permissions,
                    options,
                )
                .or_else(|| lower.ref_at(lvl2_index, lvl2).flatten())
            };
            let lvl4 = |entry: &PageEntryLvl4, lower: &SafePageMapLvl3| {
                check_perms(
                    entry.is_present_set(),
                    entry.get_permissions(),
                    permissions,
                    options,
                )
                .or_else(|| lower.ref_at(lvl3_index, lvl3).flatten())
            };

            ro_checks.read().ref_at(lvl4_index, lvl4).flatten()
        }) {
            return Err(err);
        }

        // Now that we are sure that our permissions/options are *mostly* correct, we
        // can now perform the expensive locking.
        //
        // If we are the loaded table, we need to keep track of our phys ptrs and go back
        // to write them. This is to prevent a deadlock with writing the table.
        let is_loaded = self.is_loaded();
        let mut vaddr3: Option<VirtAddr> = None;
        let mut vaddr2: Option<VirtAddr> = None;
        let mut vaddr1: Option<VirtAddr> = None;

        // make sure to get back the ro_lock
        let (ro_lock, prev_page) = {
            // Ensure an entry exists
            let lock = match lock.as_ref() {
                None => {
                    let mut upgraded_lock = lock.upgrade();
                    *upgraded_lock = Some(Arc::new(RwLock::new(SafePageMapLvl4::empty())));
                    upgraded_lock.downgrade_to_upgradeable()
                }
                _ => lock,
            };

            let mut lvl4_mut = lock.as_ref().unwrap().write();

            let lvl2_fun = |entry: &mut PageEntryLvl2, table: &mut SafePageMapLvl1| {
                let prev_permissions = entry.get_permissions();

                // If this is a new entry and we are currently loaded, we save this addr for later
                if is_loaded && !entry.is_present_set() {
                    vaddr1 = Some(VirtAddr::new(table.table.table_ptr() as usize));
                }
                // If we are not currently loaded, and this is a new entry, we are safe to gather its
                // PhysAddr.
                else if !is_loaded && !entry.is_present_set() {
                    entry.set_next_entry_phy_address(
                        VirtAddr::new(table.table.table_ptr() as usize)
                            .phys_addr()
                            .map_err(|perr| PageCorrelationError::PhysTranslationErr(perr))?
                            .addr() as u64,
                    );
                }

                // Zero entry if its not present
                if !entry.is_present_set() {
                    entry.add_permissions_from(permissions);
                } else {
                    // Unless we are told to upgrade permissions, fail
                    if prev_permissions < permissions && !options.is_increase_perm_set() {
                        return Err(PageCorrelationError::ExistingPermissionsTooStrict {
                            table_perms: entry.get_permissions(),
                            requested_perms: permissions,
                        });
                    } else if options.is_increase_perm_set() {
                        entry.add_permissions_from(permissions);
                    }

                    // Unless we are told to reduce permissions, fail
                    if prev_permissions > permissions
                        && !options.is_reduce_perm_from_tables_set()
                        && !options.is_overwrite_set()
                    {
                        return Err(PageCorrelationError::ExistingPermissionsPermissive {
                            table_perms: entry.get_permissions(),
                            requested_perms: permissions,
                        });
                    } else if prev_permissions < permissions
                        && options.is_reduce_perm_from_tables_set()
                        && options.is_overwrite_set()
                    {
                        entry.reduce_permissions_to(permissions);
                    }
                }

                // Set this flag to present if we have any flags enabled
                entry.set_present_flag(permissions.0 != 0);

                // Otherwise this isnt a new entry, and we don't care about finding its PhysAddr
                table.inner_correlate_page(lvl1_index, vpage, ppage, options, permissions)
            };

            let lvl3_fun = |entry: &mut PageEntryLvl3, table: &mut SafePageMapLvl2| {
                let prev_permissions = entry.get_permissions();

                // If this is a new entry and we are currently loaded, we save this addr for later
                if is_loaded && !entry.is_present_set() {
                    vaddr2 = Some(VirtAddr::new(table.table.table_ptr() as usize));
                }
                // If we are not currently loaded, and this is a new entry, we are safe to gather its
                // PhysAddr.
                else if !is_loaded && !entry.is_present_set() {
                    entry.set_next_entry_phy_address(
                        VirtAddr::new(table.table.table_ptr() as usize)
                            .phys_addr()
                            .map_err(|perr| PageCorrelationError::PhysTranslationErr(perr))?
                            .addr() as u64,
                    );
                }

                // Zero entry if its not present
                if !entry.is_present_set() {
                    entry.add_permissions_from(permissions);
                } else {
                    // Unless we are told to upgrade permissions, fail
                    if prev_permissions < permissions && !options.is_increase_perm_set() {
                        return Err(PageCorrelationError::ExistingPermissionsTooStrict {
                            table_perms: entry.get_permissions(),
                            requested_perms: permissions,
                        });
                    } else if options.is_increase_perm_set() {
                        entry.add_permissions_from(permissions);
                    }

                    // Unless we are told to reduce permissions, fail
                    if prev_permissions > permissions
                        && !options.is_reduce_perm_from_tables_set()
                        && !options.is_overwrite_set()
                    {
                        return Err(PageCorrelationError::ExistingPermissionsPermissive {
                            table_perms: entry.get_permissions(),
                            requested_perms: permissions,
                        });
                    } else if prev_permissions < permissions
                        && options.is_reduce_perm_from_tables_set()
                        && options.is_overwrite_set()
                    {
                        entry.reduce_permissions_to(permissions);
                    }
                }

                // Set this flag to present if we have any flags enabled
                entry.set_present_flag(permissions.0 != 0);

                // Otherwise this isnt a new entry, and we don't care about finding its PhysAddr
                table.ensured_mut_at(lvl2_index, lvl2_fun)
            };

            let prev_page = lvl4_mut.ensured_mut_at(lvl4_index, |entry, table| {
                let prev_permissions = entry.get_permissions();

                // If this is a new entry and we are currently loaded, we save this addr for later
                if is_loaded && !entry.is_present_set() {
                    vaddr3 = Some(VirtAddr::new(table.table.table_ptr() as usize));
                }
                // If we are not currently loaded, and this is a new entry, we are safe to gather its
                // PhysAddr.
                else if !is_loaded && !entry.is_present_set() {
                    entry.set_next_entry_phy_address(
                        VirtAddr::new(table.table.table_ptr() as usize)
                            .phys_addr()
                            .map_err(|perr| PageCorrelationError::PhysTranslationErr(perr))?
                            .addr() as u64,
                    );
                }

                // Zero entry if its not present
                if !entry.is_present_set() {
                    entry.add_permissions_from(permissions);
                } else {
                    // Unless we are told to upgrade permissions, fail
                    if prev_permissions < permissions && !options.is_increase_perm_set() {
                        return Err(PageCorrelationError::ExistingPermissionsTooStrict {
                            table_perms: entry.get_permissions(),
                            requested_perms: permissions,
                        });
                    } else if options.is_increase_perm_set() {
                        entry.add_permissions_from(permissions);
                    }

                    // Unless we are told to reduce permissions, fail
                    if prev_permissions > permissions
                        && !options.is_reduce_perm_from_tables_set()
                        && !options.is_overwrite_set()
                    {
                        return Err(PageCorrelationError::ExistingPermissionsPermissive {
                            table_perms: entry.get_permissions(),
                            requested_perms: permissions,
                        });
                    } else if prev_permissions < permissions
                        && options.is_reduce_perm_from_tables_set()
                        && options.is_overwrite_set()
                    {
                        entry.reduce_permissions_to(permissions);
                    }
                }

                // Set this flag to present if we have any flags enabled
                entry.set_present_flag(permissions.0 != 0);

                // Otherwise this isnt a new entry, and we don't care about finding its PhysAddr
                table.ensured_mut_at(lvl3_index, lvl3_fun)
            })?;

            drop(lvl4_mut);
            (lock, prev_page)
        };

        // If we are loaded and one of the page tables was just now created,
        // we need to write back their physical addresses.
        if is_loaded && (vaddr1.is_some() || vaddr2.is_some() || vaddr3.is_some()) {
            // Convert addresses now that we released the lock
            let paddr1 = match vaddr1 {
                Some(vaddr1) => Some(
                    vaddr1
                        .phys_addr()
                        .map_err(|perr| PageCorrelationError::PhysTranslationErr(perr))?,
                ),
                None => None,
            };
            let paddr2 = match vaddr2 {
                Some(vaddr2) => Some(
                    vaddr2
                        .phys_addr()
                        .map_err(|perr| PageCorrelationError::PhysTranslationErr(perr))?,
                ),
                None => None,
            };
            let paddr3 = match vaddr3 {
                Some(vaddr3) => Some(
                    vaddr3
                        .phys_addr()
                        .map_err(|perr| PageCorrelationError::PhysTranslationErr(perr))?,
                ),
                None => None,
            };

            // Re-acquire the 'R/W' lock
            let mut lvl4_mut = ro_lock.as_ref().unwrap().write();

            // Write back the physical address that we calculated
            lvl4_mut.ensured_mut_at(lvl4_index, |entry, table| {
                if let Some(paddr3) = paddr3 {
                    entry.set_next_entry_phy_address(paddr3.addr() as u64);
                }

                table.ensured_mut_at(lvl3_index, |entry, table| {
                    if let Some(paddr2) = paddr2 {
                        entry.set_next_entry_phy_address(paddr2.addr() as u64);
                    }

                    table.ensured_mut_at(lvl2_index, |entry, _| {
                        if let Some(paddr1) = paddr1 {
                            entry.set_next_entry_phy_address(paddr1.addr() as u64);
                        }
                    })
                })
            });
        }

        // Finally we are done :)
        Ok(prev_page)
    }
}

/// Options for mapping a page
#[make_hw(
    /// If there is already a mapped page here, override it
    field(RW, 0, pub overwrite),
    /// If you are mapping a page entry with permissive options (say the USER bit)
    /// the page tables that map it will also need that bit set. This enables that
    /// bit to be set on higher level tables.
    ///
    /// If `REDUCE_PERM` is also set, it will reduce permissions of above tables.
    field(RW, 1, pub increase_perm),
    /// Try to lock if possible, but fail if the lock is already taken.
    field(RW, 2, pub dont_wait_for_lock),
    /// Don't flush 'Tlb' cache when mapping this page
    field(RW, 3, pub no_tlb_flush),
    /// Reduce permissions to new permissions (only on tables, not page entries).
    field(RW, 4, pub reduce_perm_from_tables),
    /// Just change permissions, dont change page mapping
    field(RW, 5, pub only_commit_permissions),
    /// Force set permissions, regardless if they are higher or lower for the bottom most table only
    field(RW, 6, pub force_permissions_on_page),
)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VmOptions(usize);

/// Permissions for mapping a page
#[make_hw(
    /// Make execute is possible
    field(RW, 0, pub exec),
    /// Make reading possible
    field(RW, 1, pub read),
    /// Make writing possible
    field(RW, 2, pub write),
    /// Make userspace accessable
    field(RW, 3, pub user)
)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VmPermissions(u8);

impl VmOptions {
    /// No Options
    pub const fn none() -> Self {
        Self(0)
    }
}

impl Add for VmPermissions {
    type Output = VmPermissions;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl AddAssign for VmPermissions {
    fn add_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl core::fmt::Debug for VmPermissions {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VmPermissions")
            .field("exec", &self.is_exec_set())
            .field("read", &self.is_read_set())
            .field("write", &self.is_write_set())
            .field("user", &self.is_user_set())
            .finish()
    }
}

impl core::fmt::Display for VmPermissions {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "(VmPermissions: {}{}{}{})",
            self.is_read_set().then_some("P").unwrap_or("-"),
            self.is_write_set().then_some("W").unwrap_or("R"),
            self.is_exec_set().then_some("X").unwrap_or("-"),
            self.is_user_set().then_some("U").unwrap_or("S")
        )
    }
}

impl VmPermissions {
    /// No Permissions
    pub const NONE: VmPermissions = VmPermissions(0);
    /// Make execute is possible
    pub const EXEC: VmPermissions = VmPermissions(1 << 0);
    /// Make reading possible
    pub const READ: VmPermissions = VmPermissions(1 << 1);
    /// Make writing possible
    pub const WRITE: VmPermissions = VmPermissions(1 << 2);
    /// Make userspace accessable
    pub const USER: VmPermissions = VmPermissions(1 << 3);

    /// No permissions
    pub const fn none() -> Self {
        Self(0)
    }
}

impl BitOr for VmPermissions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for VmPermissions {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl PartialOrd for VmPermissions {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        if self.is_exec_set() && !other.is_exec_set() {
            return Some(core::cmp::Ordering::Less);
        }
        if self.is_read_set() && !other.is_read_set() {
            return Some(core::cmp::Ordering::Less);
        }
        if self.is_write_set() && !other.is_write_set() {
            return Some(core::cmp::Ordering::Less);
        }
        if self.is_user_set() && !other.is_user_set() {
            return Some(core::cmp::Ordering::Less);
        }

        if self.0 == other.0 {
            return Some(core::cmp::Ordering::Equal);
        }

        Some(core::cmp::Ordering::Greater)
    }
}
impl Ord for VmPermissions {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PageCorrelationError {
    PhysTranslationErr(PhysPtrTranslationError),
    /// This page was already mapped, and the 'OVERRIDE' flag was not set
    PageAlreadyMapped,
    /// The above table didn't have the required flags, and the 'REC_PERM' flag was not set
    ExistingPermissionsTooStrict {
        table_perms: VmPermissions,
        requested_perms: VmPermissions,
    },
    /// The above table didn't have the required flags, and the 'REC_PERM' flag was not set
    ExistingPermissionsPermissive {
        table_perms: VmPermissions,
        requested_perms: VmPermissions,
    },
    /// The table was already locked, and the 'NO_WAIT_FOR_LOCK' flag was set
    AlreadyLocked,
}

/// A trait to intoduce permissions into page table entries
pub trait PagePermissions {
    fn reduce_permissions_to(&mut self, permissions: VmPermissions);
    fn add_permissions_from(&mut self, permissions: VmPermissions);
    fn get_permissions(&self) -> VmPermissions;
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
                fn get_permissions(&self) -> VmPermissions {
                    VmPermissions::none()
                        .set_exec_flag(!self.is_execute_disable_set())
                        .set_write_flag(self.is_read_write_set())
                        .set_user_flag(self.is_user_access_set())
                        .set_read_flag(self.is_present_set())
                }
            }
        )*
    };
}

page_permissions_for! { PageEntry1G, PageEntry2M, PageEntry4K, PageEntryLvl2, PageEntryLvl3, PageEntryLvl4 }

impl SafePageMapLvl4 {
    /// Make an empty page table mapping
    const fn empty() -> Self {
        Self {
            table: PageMapLvl4::new(),
            lower: [const { None }; 512],
        }
    }

    /// Ensure a page table exists, then run F
    ///
    /// This will return the page table at index regardless if its allocated or not.
    fn ensured_mut_at<F, R>(&mut self, index: usize, f: F) -> R
    where
        F: FnOnce(&mut PageEntryLvl4, &mut SafePageMapLvl3) -> R,
    {
        assert!(
            index < 512,
            "Table index ({index}) was out of range for a PageTable (max=512)!"
        );
        let is_zero = self.lower[index].is_none();
        let table_mut = self.lower[index].get_or_insert_with(|| Box::new(SafePageMapLvl3::empty()));
        let mut entry_mut = if is_zero {
            PageEntryLvl4::zero()
        } else {
            self.table.get(index)
        };

        let reponse = f(&mut entry_mut, table_mut);

        self.table.store(entry_mut, index);

        reponse
    }

    /// If the table exists then run F
    fn ref_at<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(&PageEntryLvl4, &SafePageMapLvl3) -> R,
    {
        assert!(
            index < 512,
            "Table index ({index}) was out of range for a PageTable (max=512)!"
        );

        let entry = self.table.get(index);

        match self.lower[index] {
            Some(ref table_ref) => Some(f(&entry, table_ref)),
            None => None,
        }
    }
}

impl SafePageMapLvl3 {
    /// Make an empty page table mapping
    const fn empty() -> Self {
        Self {
            table: PageMapLvl3::new(),
            lower: [const { None }; 512],
        }
    }

    /// Ensure a page table exists, then run F
    ///
    /// This will return the page table at index regardless if its allocated or not.
    fn ensured_mut_at<F, R>(&mut self, index: usize, f: F) -> R
    where
        F: FnOnce(&mut PageEntryLvl3, &mut SafePageMapLvl2) -> R,
    {
        assert!(
            index < 512,
            "Table index ({index}) was out of range for a PageTable (max=512)!"
        );
        let is_zero = self.lower[index].is_none();
        let table_mut = self.lower[index].get_or_insert_with(|| Box::new(SafePageMapLvl2::empty()));
        let mut entry_mut = if is_zero {
            PageEntryLvl3::zero()
        } else {
            self.table.get(index)
        };

        let reponse = f(&mut entry_mut, table_mut);

        self.table.store(entry_mut, index);

        reponse
    }

    /// If the table exists then run F
    fn ref_at<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(&PageEntryLvl3, &SafePageMapLvl2) -> R,
    {
        assert!(
            index < 512,
            "Table index ({index}) was out of range for a PageTable (max=512)!"
        );

        let entry = self.table.get(index);

        match self.lower[index] {
            Some(ref table_ref) => Some(f(&entry, table_ref)),
            None => None,
        }
    }
}

impl SafePageMapLvl2 {
    /// Make an empty page table mapping
    const fn empty() -> Self {
        Self {
            table: PageMapLvl2::new(),
            lower: [const { None }; 512],
        }
    }

    /// Ensure a page table exists, then run F
    ///
    /// This will return the page table at index regardless if its allocated or not.
    fn ensured_mut_at<F, R>(&mut self, index: usize, f: F) -> R
    where
        F: FnOnce(&mut PageEntryLvl2, &mut SafePageMapLvl1) -> R,
    {
        assert!(
            index < 512,
            "Table index ({index}) was out of range for a PageTable (max=512)!"
        );
        let is_zero = self.lower[index].is_none();
        let table_mut = self.lower[index].get_or_insert_with(|| Box::new(SafePageMapLvl1::empty()));
        let mut entry_mut = if is_zero {
            PageEntryLvl2::zero()
        } else {
            self.table.get(index)
        };

        let reponse = f(&mut entry_mut, table_mut);

        self.table.store(entry_mut, index);

        reponse
    }

    /// If the table exists then run F
    fn ref_at<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(&PageEntryLvl2, &SafePageMapLvl1) -> R,
    {
        assert!(
            index < 512,
            "Table index ({index}) was out of range for a PageTable (max=512)!"
        );

        let entry = self.table.get(index);

        match self.lower[index] {
            Some(ref table_ref) => Some(f(&entry, table_ref)),
            None => None,
        }
    }
}

impl SafePageMapLvl1 {
    /// Make an empty page table mapping
    const fn empty() -> Self {
        Self {
            table: PageMapLvl1::new(),
        }
    }

    /// Do the actual inner correlating of the pages
    pub fn inner_correlate_page(
        &mut self,
        local_table_index: usize,
        vpage: VirtPage,
        ppage: PhysPage,
        options: VmOptions,
        permissions: VmPermissions,
    ) -> Result<Option<PhysPage>, PageCorrelationError> {
        let mut entry = self.table.get(local_table_index);
        let mut prev_entry = Some(entry);
        let prev_permissions = entry.get_permissions();

        // Make sure we don't override unless we are told to
        if entry.is_present_set() && !options.is_overwrite_set() {
            return Err(PageCorrelationError::PageAlreadyMapped);
        }

        // Zero entry if its not present
        if !entry.is_present_set() {
            entry = PageEntry4K::zero();
            prev_entry = None;
            entry.add_permissions_from(permissions);
        } else {
            // Unless we are told to upgrade permissions, fail
            if prev_permissions < permissions && !options.is_increase_perm_set() {
                return Err(PageCorrelationError::ExistingPermissionsTooStrict {
                    table_perms: entry.get_permissions(),
                    requested_perms: permissions,
                });
            } else if options.is_increase_perm_set() {
                entry.add_permissions_from(permissions);
            }

            // Unless we are told to reduce permissions, fail
            if prev_permissions > permissions
                && !options.is_overwrite_set()
                && !options.is_force_permissions_on_page_set()
            {
                return Err(PageCorrelationError::ExistingPermissionsPermissive {
                    table_perms: entry.get_permissions(),
                    requested_perms: permissions,
                });
            } else if prev_permissions < permissions && options.is_force_permissions_on_page_set() {
                entry.reduce_permissions_to(permissions);
            }
        }

        if !options.is_only_commit_permissions_set() {
            // do the actual linking of vpage -> ppage
            entry.set_phy_address(ppage.addr().addr() as u64);
        }

        self.table.store(entry, local_table_index);

        // Flush the TLB after storing the page
        if !options.is_no_tlb_flush_set() {
            unsafe { flush_tlb(vpage) };
        }

        Ok(prev_entry.map(|entry| {
            let addr = PhysAddr::new(entry.get_phy_address() as usize);
            PhysPage::try_from(addr.align_into::<4096>()).unwrap()
        }))
    }
}

impl core::fmt::Debug for Virt2PhysMapping {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mapping_lock = self.mapping.read();

        let Some(_) = mapping_lock.as_ref() else {
            writeln!(f, "Virt2PhysMapping ?? (Empty)")?;
            return Ok(());
        };

        write!(f, "Virt2PhysMapping (",)?;

        if self.is_loaded() {
            write!(f, "Current")?;
        } else {
            write!(
                f,
                "Not Loaded, refs={}",
                self.mapping
                    .read()
                    .as_ref()
                    .map(|inner| Arc::strong_count(inner))
                    .unwrap_or(0)
            )?;
        }
        writeln!(f, ")")
    }
}

impl core::fmt::Display for Virt2PhysMapping {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mapping_lock = self.mapping.read();

        let Some(ref inner) = mapping_lock.as_ref() else {
            writeln!(f, "Virt2PhysMapping ?? (Empty)")?;
            return Ok(());
        };

        let lvl4 = inner.read();

        write!(f, "Virt2PhysMapping (",)?;

        if self.is_loaded() {
            write!(f, "Current")?;
        } else {
            write!(
                f,
                "Not Loaded, refs={}",
                self.mapping
                    .read()
                    .as_ref()
                    .map(|inner| Arc::strong_count(inner))
                    .unwrap_or(0)
            )?;
        }
        writeln!(f, ") ::")?;

        // Formatting for LVL2 page tables
        fn lvl2_fun(
            f: &mut core::fmt::Formatter<'_>,
            skip_4k: bool,
            lvl4_index: usize,
            lvl3_index: usize,
            lvl2_index: usize,
            lvl2_entry: &PageEntryLvl2,
            lvl1: &SafePageMapLvl1,
        ) -> core::fmt::Result {
            if !lvl2_entry.is_present_set() {
                return Ok(());
            }

            writeln!(
                f,
                " | | |-{lvl2_index:>3}. PageTableLvl1 {} :: [VirtRegion {:#016x}] {}",
                lvl2_entry.get_permissions(),
                lvl4_index * (PageMapLvl4::SIZE_PER_INDEX as usize)
                    + lvl3_index * (PageMapLvl3::SIZE_PER_INDEX as usize)
                    + lvl2_index * (PageMapLvl2::SIZE_PER_INDEX as usize),
                lvl2_entry
                    .is_page_size_set()
                    .then_some("Huge Page")
                    .unwrap_or("")
            )?;

            // I use the alt-formatting option '#' to disable the 4k pages
            if !skip_4k {
                for lvl1_index in 0..512 {
                    let lvl1_entry = lvl1.table.get(lvl1_index);

                    if !lvl1_entry.is_present_set() {
                        continue;
                    }

                    writeln!(
                        f,
                        " | | | |-{lvl1_index:>3}. Page4K {} :: [VirtAddr {:#016x} -> PhysAddr {:#016x}]",
                        lvl1_entry.get_permissions(),
                        lvl4_index * (PageMapLvl4::SIZE_PER_INDEX as usize)
                            + lvl3_index * (PageMapLvl3::SIZE_PER_INDEX as usize)
                            + lvl2_index * (PageMapLvl2::SIZE_PER_INDEX as usize)
                            + lvl1_index * (PageMapLvl1::SIZE_PER_INDEX as usize),
                        lvl1_entry.get_phy_address()
                    )?;
                }
            } else {
                writeln!(f, " | | | |- <...>")?;
            }

            Ok(())
        }

        // Formatting for LVL3 page tables
        fn lvl3_fun(
            f: &mut core::fmt::Formatter<'_>,
            skip_4k: bool,
            lvl4_index: usize,
            lvl3_index: usize,
            lvl3_entry: &PageEntryLvl3,
            lvl2: &SafePageMapLvl2,
        ) -> core::fmt::Result {
            if !lvl3_entry.is_present_set() {
                return Ok(());
            }

            writeln!(
                f,
                " | |-{lvl3_index:>3}. PageTableLvl2 {} :: [VirtRegion {:#016x}] {}",
                lvl3_entry.get_permissions(),
                lvl4_index * (PageMapLvl4::SIZE_PER_INDEX as usize)
                    + lvl3_index * (PageMapLvl3::SIZE_PER_INDEX as usize),
                lvl3_entry
                    .is_page_size_set()
                    .then_some("Huge Page")
                    .unwrap_or("")
            )?;

            for lvl2_index in 0..512 {
                if let Some(err) = lvl2.ref_at(lvl2_index, |lvl2_entry, lvl1| {
                    lvl2_fun(
                        f, skip_4k, lvl4_index, lvl3_index, lvl2_index, lvl2_entry, lvl1,
                    )
                }) {
                    err?;
                }
            }

            Ok(())
        }

        // Formatting for LVL4 page tables
        fn lvl4_fun(
            f: &mut core::fmt::Formatter<'_>,
            skip_4k: bool,
            lvl4_index: usize,
            lvl4_entry: &PageEntryLvl4,
            lvl3: &SafePageMapLvl3,
        ) -> core::fmt::Result {
            if !lvl4_entry.is_present_set() {
                return Ok(());
            }

            writeln!(
                f,
                " |-{lvl4_index:>3}. PageTableLvl3 {} :: [VirtRegion {:#016x}] {}",
                lvl4_entry.get_permissions(),
                lvl4_index * (PageMapLvl4::SIZE_PER_INDEX as usize),
                lvl4_entry
                    .is_page_size_set()
                    .then_some("Huge Page")
                    .unwrap_or(""),
            )?;

            for lvl3_index in 0..512 {
                if let Some(err) = lvl3.ref_at(lvl3_index, |lvl3_entry, lvl2| {
                    lvl3_fun(f, skip_4k, lvl4_index, lvl3_index, lvl3_entry, lvl2)
                }) {
                    err?;
                }
            }

            Ok(())
        }

        for lvl4_index in 0..512 {
            if let Some(err) = lvl4.ref_at(lvl4_index, |lvl4_entry, lvl3| {
                lvl4_fun(f, f.alternate(), lvl4_index, lvl4_entry, lvl3)
            }) {
                err?;
            }
        }

        Ok(())
    }
}
