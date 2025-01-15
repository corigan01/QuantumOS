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

use core::{
    fmt::Debug,
    ops::{Add, AddAssign, BitOr, BitOrAssign, Sub, SubAssign},
};

use crate::{MemoryError, pmm::PhysPage};
use alloc::{boxed::Box, collections::btree_map::BTreeMap, vec::Vec};
use hw::make_hw;
use lldebug::logln;
use spin::RwLock;
use util::consts::{PAGE_1G, PAGE_2M, PAGE_4K};

extern crate alloc;

pub mod backing;
mod page;

pub struct VmPageIter {
    start_page: VirtPage,
    end_page: VirtPage,
}

impl Iterator for VmPageIter {
    type Item = VirtPage;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end_page >= self.start_page {
            let result = Some(self.start_page);
            self.start_page.0 += 1;

            result
        } else {
            None
        }
    }
}

pub struct VmPageLargest2MIter {
    start_page: VirtPage,
    end_page: VirtPage,
}

impl Iterator for VmPageLargest2MIter {
    type Item = VmContinuousPageArea;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end_page >= self.start_page {
            if self.start_page.0 + (PAGE_2M / PAGE_4K) <= self.end_page.0 {
                self.start_page.0 += PAGE_2M / PAGE_4K;
                Some(VmContinuousPageArea::Area2M(self.start_page))
            } else {
                self.start_page.0 += 1;
                Some(VmContinuousPageArea::Area4K(self.start_page))
            }
        } else {
            None
        }
    }
}

pub struct VmPageLargest1GIter {
    start_page: VirtPage,
    end_page: VirtPage,
}

impl Iterator for VmPageLargest1GIter {
    type Item = VmContinuousPageArea;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end_page >= self.start_page {
            if self.start_page.0 + (PAGE_1G / PAGE_4K) <= self.end_page.0 {
                self.start_page.0 += PAGE_1G / PAGE_4K;
                Some(VmContinuousPageArea::Area1G(self.start_page))
            } else if self.start_page.0 + (PAGE_2M / PAGE_4K) <= self.end_page.0 {
                self.start_page.0 += PAGE_2M / PAGE_4K;
                Some(VmContinuousPageArea::Area2M(self.start_page))
            } else {
                self.start_page.0 += 1;
                Some(VmContinuousPageArea::Area4K(self.start_page))
            }
        } else {
            None
        }
    }
}

pub enum VmContinuousPageArea {
    Area4K(VirtPage),
    Area2M(VirtPage),
    Area1G(VirtPage),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct VmRegion {
    pub start: VirtPage,
    pub end: VirtPage,
}

impl VmRegion {
    /// Is this vaddr within the page bounds.
    pub const fn contains_vaddr(&self, vaddr: u64) -> bool {
        vaddr as usize >= (self.start.0 * PAGE_4K) && (self.end.0 * PAGE_4K) >= vaddr as usize
    }
    ///
    /// Is this vaddr within the page bounds.
    pub const fn contains_vpage(&self, vaddr: VirtPage) -> bool {
        vaddr.0 >= self.start.0 && self.end.0 >= vaddr.0
    }

    pub const fn from_vaddr(vaddr: u64, len: usize) -> Self {
        let start = VirtPage::containing_page(vaddr);
        let end = VirtPage::containing_page(vaddr + len as u64);

        Self { start, end }
    }

    pub const fn iter_4k(&self) -> VmPageIter {
        VmPageIter {
            start_page: self.start,
            end_page: self.end,
        }
    }

    pub const fn iter_2m(&self) -> VmPageLargest2MIter {
        VmPageLargest2MIter {
            start_page: self.start,
            end_page: self.end,
        }
    }

    pub const fn iter_1g(&self) -> VmPageLargest1GIter {
        VmPageLargest1GIter {
            start_page: self.start,
            end_page: self.end,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct VirtPage(pub usize);

impl Sub for VirtPage {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for VirtPage {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Add for VirtPage {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for VirtPage {
    fn add_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl VirtPage {
    /// Get the page that contains this virtal address.
    pub const fn containing_page(vaddr: u64) -> VirtPage {
        VirtPage(vaddr as usize / PAGE_4K)
    }
}

#[make_hw(
    field(RW, 0, exec),
    field(RW, 1, read),
    field(RW, 2, write),
    field(RW, 3, user)
)]
#[derive(Clone, Copy)]
pub struct VmPermissions(u8);

impl Debug for VmPermissions {
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
    pub const NONE: VmPermissions = VmPermissions(0);
    pub const EXEC: VmPermissions = VmPermissions(1 << 0);
    pub const READ: VmPermissions = VmPermissions(1 << 1);
    pub const WRITE: VmPermissions = VmPermissions(1 << 2);
    pub const USER: VmPermissions = VmPermissions(1 << 3);

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

/// Manages and controls a VmObject
#[derive(Debug)]
struct VmBorderObject {
    object: Box<dyn backing::VmRegionObject>,
    mapping: BTreeMap<VirtPage, PhysPage>,
}

impl VmBorderObject {
    pub fn new(object: Box<dyn backing::VmRegionObject>) -> Self {
        Self {
            object,
            mapping: BTreeMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct VmProcess {
    objects: RwLock<Vec<VmBorderObject>>,
    page_table: page::SharedTable,
}

impl VmProcess {
    pub unsafe fn new_from_bootloader() -> Self {
        Self {
            objects: RwLock::new(Vec::new()),
            page_table: unsafe { page::SharedTable::new_from_bootloader() },
        }
    }

    pub fn new_from(other: &VmProcess) -> Self {
        Self {
            objects: RwLock::new(Vec::new()),
            page_table: other.page_table.clone(),
        }
    }

    pub fn add_vm_object(&self, object: Box<dyn backing::VmRegionObject>) {
        self.objects.write().push(VmBorderObject::new(object));
    }

    pub unsafe fn load_page_tables(&self) -> Result<(), MemoryError> {
        unsafe { self.page_table.load() }
    }

    pub fn map_all_now(&self) -> Result<(), MemoryError> {
        let mut objects = self.objects.write();

        // First load all pages into page tables
        for b_obj in objects.iter_mut() {
            let vm_region = b_obj.object.vm_region();

            // Kernel Priv
            let vm_permissions = VmPermissions::READ | VmPermissions::WRITE;

            for vpage in vm_region.iter_4k() {
                let ppage = b_obj.object.alloc_phys_anywhere()?;
                self.page_table.map_4k_page(vpage, ppage, vm_permissions)?;
                b_obj.mapping.insert(vpage, ppage);
            }
        }

        logln!("{}", self.page_table);
        unsafe { self.page_table.load()? };
        logln!("{}", self.page_table);

        for b_obj in objects.iter_mut() {
            let vm_region = b_obj.object.vm_region();
            let vm_permissions = b_obj.object.vm_permissions();

            for vpage in vm_region.iter_4k() {
                let ppage = *b_obj.mapping.get(&vpage).ok_or(MemoryError::NotFound)?;
                b_obj.object.init_page(vpage, ppage)?;

                self.page_table.map_4k_page(vpage, ppage, vm_permissions)?;
            }
        }

        unsafe { self.page_table.load() }
    }

    pub fn dump_page_tables(&self) {
        logln!("{}", self.page_table);
    }
}
