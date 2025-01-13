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

use core::ops::{BitOr, BitOrAssign};

use crate::{MemoryError, pmm::PhysPage};
use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};
use arch::idt64::{InterruptFlags, InterruptInfo};
use hw::make_hw;
use lldebug::logln;
use util::consts::PAGE_4K;

extern crate alloc;

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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VmRegion {
    pub start: VirtPage,
    pub end: VirtPage,
}

impl VmRegion {
    /// Is this vaddr within the page bounds.
    pub const fn contains_vaddr(&self, vaddr: u64) -> bool {
        vaddr as usize >= (self.start.0 * PAGE_4K) && (self.end.0 * PAGE_4K) >= vaddr as usize
    }

    pub const fn iter(&self) -> VmPageIter {
        VmPageIter {
            start_page: self.start,
            end_page: self.end,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPage(pub usize);

impl VirtPage {
    /// Get the page that contains this virtal address.
    pub const fn containing_page(vaddr: u64) -> VirtPage {
        VirtPage(vaddr as usize / PAGE_4K)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmBackingKind {
    Physical,
    Other(String),
}

trait VmBacking {
    /// The type of memory backing this structure.
    fn backing_kind(&self) -> VmBackingKind;

    /// Get the physical pages that currently exist (in memory)
    fn alive_physical_pages(&self) -> Result<Vec<PhysPage>, MemoryError>;

    /// Allocate a 'page' which is backed in physical memory.
    fn alloc_anywhere(&mut self) -> Result<PhysPage, MemoryError>;

    /// Allocate a 'page' which is backed in physical memory at a given offset.
    ///
    /// For tasks like ELF loading or INode backed memory, this is used to load
    /// a page from the file into memory.
    ///
    /// - `request`  This is the given offset (in pages) for which the allocation
    ///              would like to take place.
    ///
    ///              - If this is a file, this is the page offset in the file.
    ///              - If this is physical memory, this should be the requested
    ///                `PhysPage`.
    fn alloc_here(&mut self, request: usize) -> Result<PhysPage, MemoryError>;

    /// Deallocate a 'page' which is backed in physical memory.
    ///
    /// If this is a INode backed physical page, it may be enough to just
    /// unlink the physical page.
    fn dealloc(&mut self, page: PhysPage) -> Result<(), MemoryError>;

    /// Free all in-use physical pages.
    fn free_all(&mut self) -> Result<(), MemoryError> {
        for page in self.alive_physical_pages()? {
            self.dealloc(page)?;
        }

        Ok(())
    }

    // TODO: Maybe have a `upgrade()` and `downgrade()` function which
    //       could say convert this memory backing to a Swap if its Ram, etc...
}

#[make_hw(
    field(RW, 0, exec),
    field(RW, 1, read),
    field(RW, 2, write),
    field(RW, 3, user)
)]
#[derive(Clone, Copy)]
struct VmPermissions(u8);

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
    backing: Box<dyn VmBacking>,
    permissions: VmPermissions,
    what: String,
    // FIXME: We shouldn't create a page table for each process, we should only create the entries we need
    //        but for now this works.
    //
    // NOTE: Since `VmSafePageTable` internally is an `Arc`, this is just a ptr not an entire copy. The
    //       `VmProcess` contains the actual refrence.
    page_table: page::VmSafePageTable,
}

impl VmObject {
    /// Link this physical page to this virtal page.
    fn hydrate_page(&mut self, vpage: VirtPage, ppage: PhysPage) -> Result<(), MemoryError> {
        self.page_table.map_page(vpage, ppage)
    }

    /// Called upon PageFault. Required to lazy allocate pages.
    fn fault_handler(&mut self, info: &InterruptInfo) -> Result<(), MemoryError> {
        let InterruptFlags::PageFault {
            present, virt_addr, ..
        } = info.flags
        else {
            return Err(MemoryError::DidNotHandleException);
        };

        assert_eq!(
            self.backing.backing_kind(),
            VmBackingKind::Physical,
            "TODO: Currently we only support Physical Page backing!"
        );

        // This is not our page, we cannot handle it!
        if !self.region.contains_vaddr(virt_addr) {
            return Err(MemoryError::DidNotHandleException);
        }

        // This page does not exist, make it!
        if present {
            let ppage = self.backing.alloc_anywhere()?;
            self.hydrate_page(VirtPage::containing_page(virt_addr), ppage)?;

            Ok(())
        } else {
            Err(MemoryError::DidNotHandleException)
        }
    }

    fn vm_pages(&self) -> impl Iterator<Item = VirtPage> {
        self.region.iter()
    }
}

struct VmProcess {
    vm_process_id: usize,
    objects: Vec<VmObject>,
    page_table: page::VmSafePageTable,
}

pub struct Vmm {
    active_process: usize,
    table: BTreeMap<usize, VmProcess>,
}

impl Vmm {
    pub const KERNEL_PROCESS: usize = 0;

    pub const fn new() -> Self {
        Self {
            active_process: Self::KERNEL_PROCESS,
            table: BTreeMap::new(),
        }
    }

    pub fn init_kernel_process(
        &mut self,
        kernel_regions: impl Iterator<Item = VmRegion>,
    ) -> Result<(), MemoryError> {
        let page_tables = page::VmSafePageTable::copy_from_bootloader();
        unsafe { page_tables.load() };

        let mut vm_objects = Vec::new();
        vm_objects.push(VmObject {
            region: todo!(),
            backing: todo!(),
            permissions: todo!(),
            page_table: page_tables,
            what: todo!(),
        });

        self.table.insert(Self::KERNEL_PROCESS, VmProcess {
            vm_process_id: 0,
            objects: vm_objects,
            page_table: page_tables,
        });

        unsafe { page_tables.load() };
        todo!()
    }
}
