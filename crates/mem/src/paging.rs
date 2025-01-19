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
use core::ops::{BitOr, BitOrAssign};

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
use spin::RwLock;

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
    todo!()
}

static CURRENTLY_LOADED_PAGE_TABLES: Virt2PhysMapping = Virt2PhysMapping::empty();

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
        self.mapping.read().as_ref().is_some_and(|our_table| {
            CURRENTLY_LOADED_PAGE_TABLES
                .mapping
                .read()
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

    /// Map this `vpage` to `ppage` returning the previous PhysPage if there was one
    pub fn correlate_page(
        &self,
        vpage: VirtPage,
        ppage: PhysPage,
        options: VmOptions,
        permissions: VmPermissions,
    ) -> Result<Option<PhysPage>, PageCorrelationError> {
        if options.is_o_no_wait_for_lock_set() {}
        todo!()
    }
}

/// Options for mapping a page
#[make_hw(
    /// If there is already a mapped page here, override it
    field(RW, 0, o_override),
    /// If you are mapping a page entry with permissive options (say the USER bit)
    /// the page tables that map it will also need that bit set. This enables that
    /// bit to be set on higher level tables.
    ///
    /// If `O_REDUCE_PERM` is also set, it will reduce permissions of above tables.
    field(RW, 1, o_check_perm),
    /// Try to lock if possible, but fail if the lock is already taken.
    field(RW, 2, o_no_wait_for_lock),
    /// Don't flush 'Tlb' cache when mapping this page
    field(RW, 3, o_no_flush),
    /// Reduce permissions to new permissions.
    field(RW, 4, o_reduce_perm),
)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VmOptions(usize);

/// Permissions for mapping a page
#[make_hw(
    /// Make execute is possible
    field(RW, 0, exec),
    /// Make reading possible
    field(RW, 1, read),
    /// Make writing possible
    field(RW, 2, write),
    /// Make userspace accessable
    field(RW, 3, user)
)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VmPermissions(u8);

impl VmOptions {
    /// If there is already a mapped page here, override it
    pub const O_OVERRIDE: VmOptions = VmOptions(1 << 0);
    /// If you are mapping a page entry with permissive options (say the USER bit)
    /// the page tables that map it will also need that bit set. This enables that
    /// bit to be set on higher level tables.
    ///
    /// If `O_REDUCE_PERM` is also set, it will reduce permissions of above tables.
    pub const O_CHECK_PERM: VmOptions = VmOptions(1 << 1);
    /// Try to lock if possible, but fail if the lock is already taken.
    pub const O_NO_WAIT_FOR_LOCK: VmOptions = VmOptions(1 << 2);
    /// Don't flush 'Tlb' cache when mapping this page
    pub const O_NO_FLUSH: VmOptions = VmOptions(1 << 3);
    /// Reduce permissions to new permissions.
    pub const O_REDUCE_PERM: VmOptions = VmOptions(1 << 4);

    /// No Options
    pub const fn none() -> Self {
        Self(0)
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
        self.0.count_ones().partial_cmp(&other.0.count_ones())
    }
}
impl Ord for VmPermissions {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.count_ones().cmp(&other.0.count_ones())
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
        let table_mut = self.lower[index].get_or_insert_with(|| Box::new(SafePageMapLvl3::empty()));
        let mut entry_mut = self.table.get(index);

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
        let table_mut = self.lower[index].get_or_insert_with(|| Box::new(SafePageMapLvl2::empty()));
        let mut entry_mut = self.table.get(index);

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
        let table_mut = self.lower[index].get_or_insert_with(|| Box::new(SafePageMapLvl1::empty()));
        let mut entry_mut = self.table.get(index);

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
        if entry.is_present_set() && !options.is_o_override_set() {
            return Err(PageCorrelationError::PageAlreadyMapped);
        }

        // Zero entry if its not present
        if !entry.is_present_set() {
            entry = PageEntry4K::zero();
            prev_entry = None;
        } else {
            // Unless we are told to upgrade permissions, fail
            if prev_permissions < permissions && !options.is_o_check_perm_set() {
                return Err(PageCorrelationError::ExistingPermissionsTooStrict {
                    table_perms: entry.get_permissions(),
                    requested_perms: permissions,
                });
            } else if prev_permissions < permissions && options.is_o_check_perm_set() {
                entry.add_permissions_from(permissions);
            }

            // Unless we are told to reduce permissions, fail
            if prev_permissions > permissions && !options.is_o_reduce_perm_set() {
                return Err(PageCorrelationError::ExistingPermissionsPermissive {
                    table_perms: entry.get_permissions(),
                    requested_perms: permissions,
                });
            } else if prev_permissions < permissions && options.is_o_reduce_perm_set() {
                entry.reduce_permissions_to(permissions);
            }
        }

        // do the actual linking of vpage -> ppage
        entry.set_phy_address(ppage.addr().addr() as u64);

        self.table.store(entry, local_table_index);

        // Flush the TLB after storing the page
        if !options.is_o_no_flush_set() {
            unsafe { flush_tlb(vpage) };
        }

        Ok(prev_entry.map(|entry| {
            let addr = PhysAddr::new(entry.get_phy_address() as usize);
            PhysPage::try_from(addr.align_into::<4096>()).unwrap()
        }))
    }
}
