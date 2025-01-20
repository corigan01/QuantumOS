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
use core::{error::Error, fmt::Display};

use crate::{
    MemoryError,
    addr::{AlignedTo, VirtAddr},
    page::{Page4K, PhysPage, VirtPage},
    paging::{PageCorrelationError, Virt2PhysMapping, VmOptions, VmPermissions},
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
        self.start.addr() <= addr && (self.end.addr().offset(PAGE_4K - 1)) >= addr
    }

    /// Is this page contained within this VmRegion
    pub fn does_contain_page(&self, page: VirtPage) -> bool {
        self.start <= page && self.end >= page
    }

    /// Get an iterator of the pages contained within this region
    pub fn pages_iter(&self) -> impl Iterator<Item = VirtPage> + use<> {
        (self.start.page()..=self.end.page())
            .into_iter()
            .map(|raw_page| VirtPage::new(raw_page))
    }

    /// Does this other VmRegion overlap with our VmObject
    pub fn overlaps_with(&self, rhs: &Self) -> bool {
        self.does_contain_page(rhs.start) || self.does_contain_page(rhs.end)
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
    fn requests_all_pages_filled(&self, parent_object: &VmObject) -> bool {
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
        &self,
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

/// Scrub the vpage's memory with the given pattern.
///
/// The vpage must be kernel accessable before calling this function.
pub unsafe fn scrub_page(vpage: VirtPage, pattern: u8) {
    let slice = unsafe { core::slice::from_raw_parts_mut(vpage.addr().as_mut_ptr(), 4096) };
    slice.fill(pattern);
}

impl VmInjectFillAction for VmFillAction {
    fn populate_page(
        &mut self,
        parent_object: &VmObject,
        process: &VmProcess,
        relative_index: usize,
        vpage: VirtPage,
        ppage: PhysPage,
    ) -> PopulationReponse {
        match self {
            VmFillAction::Nothing => PopulationReponse::Okay,
            VmFillAction::Scrub(pattern) => {
                unsafe { scrub_page(vpage, *pattern) };
                PopulationReponse::Okay
            }
            VmFillAction::InjectWith(rw_lock) => {
                rw_lock
                    .write()
                    .populate_page(parent_object, process, relative_index, vpage, ppage)
            }
        }
    }

    fn requests_all_pages_filled(&self, parent_object: &VmObject) -> bool {
        match self {
            VmFillAction::Nothing => false,
            VmFillAction::Scrub(_) => false,
            VmFillAction::InjectWith(rw_lock) => {
                rw_lock.read().requests_all_pages_filled(parent_object)
            }
        }
    }

    fn page_safely_releasable(
        &self,
        parent_object: &VmObject,
        process: &VmProcess,
        vpage: VirtPage,
    ) -> bool {
        match self {
            VmFillAction::Nothing => false,
            VmFillAction::Scrub(_) => false,
            VmFillAction::InjectWith(rw_lock) => {
                rw_lock
                    .read()
                    .page_safely_releasable(parent_object, process, vpage)
            }
        }
    }

    fn page_fault_handler(
        &self,
        parent_object: &VmObject,
        process: &VmProcess,
        info: PageFaultInfo,
    ) -> PageFaultReponse {
        match self {
            // If we return with 'Handled' we will later receive a call to map that page
            //
            // We should not do the mapping of the page in the page fault handler!
            VmFillAction::Nothing => PageFaultReponse::Handled,
            VmFillAction::Scrub(_) => PageFaultReponse::Handled,
            VmFillAction::InjectWith(rw_lock) => {
                rw_lock
                    .write()
                    .page_fault_handler(parent_object, process, info)
            }
        }
    }
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
    pub fill_action: RwLock<VmFillAction>,
}

/// The type of error given when making a new page
#[derive(Debug)]
pub enum NewVmObjectError {
    /// Failed to Map this page
    MappingErr(VmObjectMappingError),
}

/// The type of error given when trying to map a page with a VmObject
#[derive(Debug)]
pub enum VmObjectMappingError {
    /// Cannot map this page
    MappingError(PageCorrelationError),
    /// Failed to get a physical page
    CannotGetPhysicalPage(MemoryError),
    /// Tried to call map_page with a page not in the region
    PageNotContainedWithinRegion {
        region: VmRegion,
        requested_vpage: VirtPage,
    },
}

impl Error for VmObjectMappingError {}
impl Display for VmObjectMappingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{:#?}", self)
    }
}

/// Options to apply to a page when populating it
const KERNEL_POPULATE_OPT: VmOptions = VmOptions::none()
    .set_overwrite_flag(true)
    .set_increase_perm_flag(true)
    .set_reduce_perm_flag(true);
/// Permissions to apply to a page when populating it
const KERNEL_POPULATE_PERM: VmPermissions = VmPermissions::none()
    .set_exec_flag(false)
    .set_read_flag(true)
    .set_write_flag(true)
    .set_user_flag(false);

impl VmObject {
    /// Create a new VmObject
    pub fn new(
        vm_process: &VmProcess,
        region: VmRegion,
        permissions: VmPermissions,
        fill_action: VmFillAction,
    ) -> Result<Arc<RwLock<Self>>, NewVmObjectError> {
        let mut new_self = Self {
            region,
            mappings: BTreeMap::new(),
            permissions,
            fill_action: RwLock::new(fill_action),
        };

        // If this region requests all of its pages to be filled now, we need to fill them
        if new_self
            .fill_action
            .read()
            .requests_all_pages_filled(&new_self)
        {
            logln!("This vmobject requests all pages filled now!");
            for vpage in new_self.region.pages_iter() {
                new_self
                    .map_new_page(vm_process, vpage)
                    .map_err(|err| NewVmObjectError::MappingErr(err))?;
            }
        }

        Ok(Arc::new(RwLock::new(new_self)))
    }

    /// Do the mapping of a virtual page
    ///
    /// This function should flush this page to the VmProcess's page tables.
    pub fn map_new_page(
        &mut self,
        vm_process: &VmProcess,
        vpage: VirtPage,
    ) -> Result<(), VmObjectMappingError> {
        logln!("Asking this VmObject to map {:?}", vpage);

        // if this page isnt within our mapping, we cannot map it
        if !self.region.does_contain_page(vpage) {
            return Err(VmObjectMappingError::PageNotContainedWithinRegion {
                region: self.region,
                requested_vpage: vpage,
            });
        }

        // Get a new backing page for this vpage
        let backing_page = SharedPhysPage::allocate_anywhere()
            .map_err(|err| VmObjectMappingError::CannotGetPhysicalPage(err))?;

        // Map the page with kernel option first to ensure we can write to this page
        vm_process
            .page_tables
            .correlate_page(
                vpage,
                *backing_page,
                KERNEL_POPULATE_OPT,
                KERNEL_POPULATE_PERM,
            )
            .map_err(|err| VmObjectMappingError::MappingError(err))?;

        // Attempt to populate the page
        match self
            .fill_action
            .write()
            .populate_page(self, vm_process, 0, vpage, *backing_page)
        {
            PopulationReponse::Okay => (),
            PopulationReponse::MappingError(page_correlation_error) => {
                return Err(VmObjectMappingError::MappingError(page_correlation_error));
            }
        }

        // Finally map the page back to the user when done
        vm_process
            .page_tables
            .correlate_page(
                vpage,
                *backing_page,
                VmOptions::none()
                    .set_only_commit_permissions_flag(true)
                    .set_increase_perm_flag(true)
                    .set_force_permissions_on_page_flag(true)
                    .set_overwrite_flag(true),
                self.permissions,
            )
            .map_err(|err| VmObjectMappingError::MappingError(err))?;

        Ok(())
    }

    /// The page fault handler for this VmObject
    pub fn page_fault_handler(
        &mut self,
        vm_process: &VmProcess,
        info: PageFaultInfo,
    ) -> PageFaultReponse {
        match self
            .fill_action
            .write()
            .page_fault_handler(self, vm_process, info)
        {
            PageFaultReponse::Handled => (),
            err => return err,
        }

        // If the FillAction returned 'Handled' we should call map_new_page() to let it allocate that page
        match self.map_new_page(vm_process, VirtPage::containing_addr(info.vaddr)) {
            Ok(_) => PageFaultReponse::Handled,
            Err(page_mapping_err) => {
                return PageFaultReponse::CriticalFault(Box::new(page_mapping_err));
            }
        }
    }
}

/// A possible reponse to inserting a VmObject into a VmProcess
#[derive(Debug)]
pub enum InsertVmObjectError {
    /// This new region overlaps with an existing region
    Overlapping {
        /// The region is overlaps with
        existing: VmRegion,
        /// The region attempted to be added
        attempted: VmRegion,
    },
    /// Thew new vm object failed
    VmObjectError(NewVmObjectError),
}

/// Repr a virtual 'Address Space' for which a processes exists in
///
/// This struct is fully locked internally, so it can be accessed via '&self'
#[derive(Debug)]
pub struct VmProcess {
    /// The objects that make up this VmProcess
    ///
    /// Since these objects can and be shared, we must lock and ref-count them
    objects: RwLock<Vec<Arc<RwLock<VmObject>>>>,
    /// The page tables in this process
    pub page_tables: Virt2PhysMapping,
}

impl VmProcess {
    /// Init an empty ProcessVM (const fn)
    pub const fn new() -> Self {
        Self {
            objects: RwLock::new(Vec::new()),
            page_tables: Virt2PhysMapping::empty(),
        }
    }

    /// Inhearit the page tables from 'page_tables'
    pub fn inhearit_page_tables(page_tables: &Virt2PhysMapping) -> Self {
        Self {
            objects: RwLock::new(Vec::new()),
            page_tables: Virt2PhysMapping::inhearit_from(page_tables),
        }
    }

    /// Does this VmRegion overlap with any of the VmObjects in this Process?
    ///
    /// If it returns the region that is overlapping.
    pub fn check_overlapping(&self, region: &VmRegion) -> Option<VmRegion> {
        self.objects.read().iter().find_map(|vm_object| {
            let locked = vm_object.read();

            if locked.region.overlaps_with(&region) {
                Some(locked.region)
            } else {
                None
            }
        })
    }

    /// Add a mapping to this process
    pub fn insert_vm_object(
        &self,
        object: Arc<RwLock<VmObject>>,
    ) -> Result<(), InsertVmObjectError> {
        let locked = object.read();

        // If there is already a region that exists on that virtual address
        if let Some(existing) = self.check_overlapping(&locked.region) {
            return Err(InsertVmObjectError::Overlapping {
                existing,
                attempted: locked.region,
            });
        }

        // Finally insert the object into the process
        drop(locked);
        self.objects.write().push(object);

        Ok(())
    }

    /// Make a new vm object from this process. This will both insert the object
    /// and return a new Arc<..> ptr to it.
    pub fn inplace_new_vmobject(
        &self,
        region: VmRegion,
        permissions: VmPermissions,
        fill_action: VmFillAction,
    ) -> Result<Arc<RwLock<VmObject>>, InsertVmObjectError> {
        // If there is already a region that exists on that virtual address
        //
        // Even though this is checked again once the object gets inserted, we
        // want to make sure this object is valid before we do the expensive work
        // of creating it.
        if let Some(existing) = self.check_overlapping(&region) {
            return Err(InsertVmObjectError::Overlapping {
                existing,
                attempted: region,
            });
        }

        // Construct the object
        let obj = VmObject::new(self, region, permissions, fill_action)
            .map_err(|obj_err| InsertVmObjectError::VmObjectError(obj_err))?;

        // Insert the object
        self.insert_vm_object(obj.clone())?;

        Ok(obj)
    }

    /// The page fault handler for this VmProcess
    pub fn page_fault_handler(&self, info: PageFaultInfo) -> PageFaultReponse {
        let lock = self.objects.read();
        let Some(object) = lock
            .iter()
            .find(|object| object.read().region.does_contain_addr(info.vaddr))
        else {
            logln!(
                "Called PageFaultHandler, but could not find a region with that addr! {:#?}",
                info
            );
            return PageFaultReponse::NotAttachedHandler;
        };

        object.write().page_fault_handler(self, info)
    }
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

// TODO: REMOVE THIS FUNCTION, its just for testing :)
pub fn test() {
    let proc = VmProcess::new();

    proc.inplace_new_vmobject(
        VmRegion::new(VirtPage::new(10), VirtPage::new(20)),
        VmPermissions::none()
            .set_read_flag(true)
            .set_write_flag(true)
            .set_user_flag(true),
        VmFillAction::Nothing,
    )
    .unwrap();

    logln!(
        "{:#?}",
        proc.page_fault_handler(PageFaultInfo {
            is_present: false,
            write_read_access: false,
            execute_fault: false,
            user_fault: false,
            vaddr: VirtPage::<Page4K>::new(15).addr(),
        })
    );

    logln!("{}", proc.page_tables);

    todo!("Test Done!");
}
