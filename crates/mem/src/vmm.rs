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
    ops::{BitOr, BitOrAssign},
};

use crate::MemoryError;
use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};
use backing::VmBacking;
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

struct VmObject {
    region: VmRegion,
    backing: Option<Box<dyn backing::VmBacking>>,
    permissions: VmPermissions,
    what: String,
}

impl Debug for VmObject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VmObject")
            .field("region", &self.region)
            .field("backing", &"...")
            .field("permissions", &self.permissions)
            .field("what", &self.what)
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum KernelRegionKind {
    KernelExe,
    KernelElf,
    KernelStack,
    KernelHeap,
}

pub struct VmProcess {
    vm_process_id: usize,
    objects: Vec<VmObject>,
    page_table: page::SharedTable,
}

impl VmProcess {
    pub unsafe fn load_page_tables(&mut self) -> Result<(), MemoryError> {
        unsafe { self.page_table.load() }
    }
}

impl Debug for VmProcess {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VmProcess")
            .field("vm_process_id", &self.vm_process_id)
            .field("objects", &self.objects)
            .field("page_table", &"...")
            .finish()
    }
}

impl VmProcess {
    fn map_all_now(&mut self) -> Result<(), MemoryError> {
        for vm_object in self.objects.iter_mut() {
            for vpage in vm_object.region.iter_4k() {
                let ppage = vm_object
                    .backing
                    .as_mut()
                    .ok_or(MemoryError::NotSupported)?
                    .alloc_anywhere()?;

                self.page_table
                    .map_4k_page(vpage, ppage, vm_object.permissions)?;
            }
        }

        Ok(())
    }

    pub fn dump_page_tables(&self) {
        logln!("{}", self.page_table);
    }
}

#[derive(Debug)]
pub struct Vmm {
    table: Vec<Arc<RwLock<VmProcess>>>,
}

impl Vmm {
    pub const fn new() -> Self {
        Self { table: Vec::new() }
    }

    pub fn init_kernel_process(
        &mut self,
        kernel_regions: impl Iterator<Item = (VmRegion, KernelRegionKind)>,
    ) -> Result<Arc<RwLock<VmProcess>>, MemoryError> {
        let page_tables = unsafe { page::SharedTable::new_from_bootloader() };
        unsafe { page_tables.load().unwrap() };

        let vm_process = Arc::new({
            let vm_objects = kernel_regions
                .map(|(region, kind)| {
                    let permissions = match kind {
                        KernelRegionKind::KernelExe => VmPermissions::EXEC | VmPermissions::READ,
                        KernelRegionKind::KernelElf => VmPermissions::READ,
                        KernelRegionKind::KernelStack => VmPermissions::READ | VmPermissions::WRITE,
                        KernelRegionKind::KernelHeap => VmPermissions::READ | VmPermissions::WRITE,
                    };

                    VmObject {
                        region,
                        backing: None,
                        permissions,
                        what: format!("{:?}", kind),
                    }
                })
                .collect();

            RwLock::new(VmProcess {
                vm_process_id: 0,
                objects: vm_objects,
                page_table: page_tables,
            })
        });

        self.table.push(vm_process.clone());
        Ok(vm_process)
    }

    fn clone_pages_from_kernel(&self) -> page::SharedTable {
        self.table[0].read().page_table.clone()
    }

    pub fn new_process<
        I: Iterator<Item = (VmRegion, VmPermissions, String, Box<dyn VmBacking>)>,
    >(
        &mut self,
        regions: I,
    ) -> Result<Arc<RwLock<VmProcess>>, MemoryError> {
        let vm_process = Arc::new({
            let vm_objects = regions
                .map(|(region, permissions, what, backing)| VmObject {
                    region,
                    backing: Some(backing),
                    permissions,
                    what,
                })
                .collect();

            RwLock::new(VmProcess {
                vm_process_id: self.table.len(),
                objects: vm_objects,
                page_table: self.clone_pages_from_kernel(),
            })
        });

        vm_process.write().map_all_now()?;
        self.table.push(vm_process.clone());
        Ok(vm_process)
    }
}
