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

use alloc::{boxed::Box, sync::Arc};
use arch::locks::InterruptMutex;
use core::ops::Deref;
use util::consts::PAGE_4K;

use crate::{
    MemoryError,
    page::PhysPage,
    phys::{PhysMemoryKind, PhysMemoryMap},
};

extern crate alloc;

mod backing;

static THE_PHYSICAL_PAGE_MANAGER: InterruptMutex<Option<Pmm>> = InterruptMutex::new(None);

pub fn set_physical_memory_manager(pmm: Pmm) {
    *THE_PHYSICAL_PAGE_MANAGER.lock() = Some(pmm);
}

pub fn use_pmm_mut<F, R>(func: F) -> R
where
    F: FnOnce(&mut Pmm) -> R,
{
    let mut pmm = THE_PHYSICAL_PAGE_MANAGER.lock();
    func(
        &mut *pmm
            .as_mut()
            .expect("Physical Memory Manager has not be set!"),
    )
}

pub fn use_pmm_ref<F, R>(func: F) -> R
where
    F: FnOnce(&Pmm) -> R,
{
    let pmm = THE_PHYSICAL_PAGE_MANAGER.lock();
    func(
        &*pmm
            .as_ref()
            .expect("Physical Memory Manager has not be set!"),
    )
}

pub struct Pmm {
    table: Box<backing::MemoryTable<backing::TableFlat>>,
}

impl Pmm {
    pub fn new<const SIZE: usize>(memory_map: &PhysMemoryMap<SIZE>) -> Result<Self, MemoryError> {
        let total_real_memory = memory_map.sdram_bytes();

        let mut opt_table = *backing::OPT_TABLES.last().unwrap();
        for table_size in backing::OPT_TABLES {
            if total_real_memory < (table_size * backing::TABLE_SIZE) {
                opt_table = table_size;
                break;
            }
        }

        let mut table = Box::new(backing::MemoryTable::new(opt_table));

        memory_map
            .iter()
            .filter(|entry| {
                entry.kind == PhysMemoryKind::Free && entry.start.addr() >= (1 * util::consts::MIB)
            })
            .try_for_each(|entry| {
                table
                    .populate_with(
                        entry.start.align_up_to(PAGE_4K).try_into().unwrap(),
                        entry.end.align_down_to(PAGE_4K).try_into().unwrap(),
                    )
                    .map(|_| ())
            })?;

        Ok(Self { table })
    }

    pub fn allocate_page(&mut self) -> Result<PhysPage, MemoryError> {
        self.table.request_page()
    }

    pub fn free_page(&mut self, page: PhysPage) -> Result<(), MemoryError> {
        self.table.free_page(page)
    }

    pub fn pages_free(&self) -> Result<usize, MemoryError> {
        self.table.pages_free()
    }
}

/// This physical page was allocated by the PMM and when dropped it
/// will automaticlly be returned. Its a refrence counted PhysicalPage.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SharedPhysPage(Arc<PhysPage>);

impl Drop for SharedPhysPage {
    fn drop(&mut self) {
        if self.ref_count() == 1 {
            use_pmm_mut(|pmm| pmm.free_page(*self.0))
                .expect("Unable to drop inner page when ref count hit zero!");
        }
    }
}

impl Deref for SharedPhysPage {
    type Target = PhysPage;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl SharedPhysPage {
    /// Allocates a new PmmPhysPage anywhere in the physical address space.
    pub fn allocate_anywhere() -> Result<Self, MemoryError> {
        let page = use_pmm_mut(|pmm| pmm.allocate_page())?;

        Ok(Self(Arc::new(page)))
    }

    /// Creates a new SharedPhysPage from a given physical page
    ///
    /// # Safety
    /// This function does not check that the page is available, so it is up to the caller
    /// to ensure the physical page is valid.
    pub unsafe fn force_new_at(page: PhysPage) -> Self {
        Self(Arc::new(page))
    }

    /// Get the refrences to this page
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }
}
