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
use core::error::Error;

use crate::{
    addr::{AlignedTo, VirtAddr},
    page::{PhysPage, VirtPage},
    paging::{PageCorrelationError, Virt2PhysMapping, VmPermissions},
    pmm::SharedPhysPage,
};
use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, vec::Vec};
use lldebug::logln;
use spin::RwLock;
use util::consts::PAGE_4K;

/// A region of virtual memory 'virtual in pages'
#[derive(Debug, Clone, Copy)]
pub struct VmRegion {
    start: VirtPage,
    end: VirtPage,
}

impl VmRegion {
    /// Create a new VmRegion from virtual pages
    pub const fn new(start: VirtPage, end: VirtPage) -> VmRegion {
        Self { start, end }
    }

    /// Create a new VmRegion from aligned virtual addresses
    pub const fn from_addr(
        start: VirtAddr<AlignedTo<PAGE_4K>>,
        end: VirtAddr<AlignedTo<PAGE_4K>>,
    ) -> Self {
        Self {
            start: VirtPage::from_aligned(start),
            end: VirtPage::from_aligned(end),
        }
    }

    /// Get the VmRegion that would contain the unaligned virtual address range
    pub const fn from_containing(start: VirtAddr, end: VirtAddr) -> Self {
        Self {
            start: VirtPage::containing_addr(start),
            end: VirtPage::containing_addr(end),
        }
    }

    /// Is this virtual address contained within this VmRegion
    pub fn does_contain_addr(&self, addr: VirtAddr) -> bool {
        self.start.addr() >= addr && (self.end.addr().offset(PAGE_4K - 1)) <= addr
    }

    /// Is this page contained within this VmRegion
    pub fn does_contain_page(&self, page: VirtPage) -> bool {
        self.start >= page && self.end <= page
    }

    // Get an iterator of the pages contained within this region
    pub fn pages_iter(&self) -> impl Iterator<Item = VirtPage> {
        (self.start.page()..=self.end.page())
            .into_iter()
            .map(|raw_page| VirtPage::new(raw_page))
    }
}

/// The reponse to a page population request
#[must_use]
#[derive(Debug)]
pub enum PopulationReponse {
    /// This request was valid and fulfilled
    Okay,
    /// There was a problem mapping this page
    MappingError(PageCorrelationError),
}

pub trait VmInjectFillAction: core::fmt::Debug {
    /// Populate this page with content from this content's provider
    fn populate_page(
        &mut self,
        parent_object: &VmObject,
        process: &VmProcess,
        relative_index: usize,
        vpage: VirtPage,
        ppage: PhysPage,
    ) -> PopulationReponse;

    /// Should all pages be filled immediately when this object is created?
    #[allow(unused_variables)]
    fn requests_all_pages_filled(&self, parent_object: &VmObject, process: &VmProcess) -> bool {
        false
    }

    /// This page has some backing that is seperate from the physical memory, and can be
    /// safely releasable.
    #[allow(unused_variables)]
    fn page_safely_releasable(
        &self,
        parent_object: &VmObject,
        process: &VmProcess,
        vpage: VirtPage,
    ) -> bool {
        false
    }

    /// What to do when this region gets a page fault (if anything)
    #[allow(unused_variables)]
    fn page_fault_handler(
        &mut self,
        parent_object: &VmObject,
        process: &VmProcess,
        info: PageFaultInfo,
    ) -> PageFaultReponse {
        // By default we don't do anything, so we reuse the 'NotAttachedHandler' to signal this
        PageFaultReponse::NotAttachedHandler
    }

    // TODO: impl a 'HandleLowMemory' which requests for this VmRegion to unback pages
    // TODO: impl a deconstructor option to delete content or flush pages
}

/// What to do with this VmObject's memory. How should it be filled?
#[derive(Debug)]
pub enum VmFillAction {
    /// Don't do anything after allocating a physical page
    Nothing,
    /// Scrub this section with a byte pattern.
    Scrub(u8),
    /// Do some more complex action with this page.
    InjectWith(Arc<RwLock<dyn VmInjectFillAction>>),
}

#[derive(Debug)]
pub struct VmObject {
    // TODO: Support VmObject sharing
    // /// Is this VmObject Shared with other processes
    // is_shared: bool,
    // /// If this VmObject is 'private' it cannot be shared
    // is_private: bool,
    // /// Supports Cow (Copy on Write pages)
    // supports_cow: bool,
    /// The region of memory this VmObject contains
    pub region: VmRegion,
    /// The physical page tables this VmObject has allocated
    pub mappings: BTreeMap<VirtAddr, SharedPhysPage>,
    /// Permissions of this object
    pub permissions: VmPermissions,
    /// What to do wiht this vm object
    pub fill_action: VmFillAction,
}

/// Repr a virtual 'Address Space' for which a processes exists in
#[derive(Debug)]
pub struct VmProcess {
    objects: Vec<VmObject>,
    page_tables: Virt2PhysMapping,
}

impl VmProcess {
    // Init an empty ProcessVM (const fn)
    pub const fn new() -> Self {
        Self {
            objects: Vec::new(),
            page_tables: Virt2PhysMapping::empty(),
        }
    }

    pub fn test(&mut self) {}
}

/// Possible scenarios for a page fault to occur
#[derive(Clone, Copy, Debug)]
pub struct PageFaultInfo {
    /// If this isnt set, the page didnt exist
    pub is_present: bool,
    /// if this flag is set, the fault was caused by a 'write' access,
    /// however, if this flag isn't set, it was caused by a 'read' access
    pub write_read_access: bool,
    /// An attempted execute was made on this page, however this page does not
    /// support execute
    pub execute_fault: bool,
    /// This page is marked 'Supervisor' but was attempted to be accessed from
    /// a 'User'
    pub user_fault: bool,
    /// The virtual address of the fault
    pub vaddr: VirtAddr,
}

/// What to do in reponse to handling a page fault
#[derive(Debug)]
pub enum PageFaultReponse {
    /// This page fault was handled
    Handled,
    /// The user does not have access to this memory
    NoAccess {
        page_perm: VmPermissions,
        request_perm: VmPermissions,
        page: VirtPage,
    },
    /// Something went wrong, and we need to panic!
    CriticalFault(Box<dyn Error>),
    /// There was no page fault handler attached
    NotAttachedHandler,
}

/// The type of function needed to attach to the system's page fault handler
type SystemAttachedPageFaultFn = fn(PageFaultInfo) -> PageFaultReponse;

/// The handler the system will call
static MAIN_PAGE_FAULT_HANDLER: RwLock<Option<SystemAttachedPageFaultFn>> = RwLock::new(None);

/// System page fault entry handler
///
/// This is the function the system is expected to call when a page fault occurs
pub fn call_page_fault_handler(info: PageFaultInfo) -> PageFaultReponse {
    // FIXME: This lock will deadlock if we fault setting the page fault handler, we
    //        should fix this in the future!
    let locked = MAIN_PAGE_FAULT_HANDLER.read();
    if let Some(locked) = locked.as_ref() {
        // call the handler function if its enabled
        locked(info)
    } else {
        // Otherwise, we tell the handler that nothing is attached
        PageFaultReponse::NotAttachedHandler
    }
}

/// Set this function to be the page fault handler
pub fn set_page_fault_handler(handler: SystemAttachedPageFaultFn) {
    *MAIN_PAGE_FAULT_HANDLER.write() = Some(handler);
}

/// Clear the function in the page fault handler, setting it to None
pub fn remove_page_fault_handler() {
    *MAIN_PAGE_FAULT_HANDLER.write() = None;
}
