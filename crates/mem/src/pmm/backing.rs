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

use crate::MemoryError;
use crate::addr::{AlignedTo, PhysAddr};
use crate::page::PhysPage;
use alloc::boxed::Box;
use boolvec::BoolVec;
use core::ptr::NonNull;
use util::consts::PAGE_4K;

pub const TABLE_SIZE: usize = 512;

pub const LVL0_TABLE: usize = PAGE_4K;
pub const LVL1_TABLE: usize = LVL0_TABLE * TABLE_SIZE;
pub const LVL2_TABLE: usize = LVL1_TABLE * TABLE_SIZE;
pub const LVL3_TABLE: usize = LVL2_TABLE * TABLE_SIZE;
pub const LVL4_TABLE: usize = LVL3_TABLE * TABLE_SIZE;

pub const OPT_TABLES: [usize; 4] = [LVL1_TABLE, LVL2_TABLE, LVL3_TABLE, LVL4_TABLE];

#[derive(Clone, Copy)]
enum TableElementKind {
    NotAllocated,
    Present,
    TableFlat {
        ptr: NonNull<MemoryTable<TableFlat>>,
        atom: usize,
    },
    TableBits {
        ptr: NonNull<MemoryTable<TableBits>>,
        atom: usize,
    },
}

pub struct TableFlat {
    table: [TableElementKind; TABLE_SIZE],
    available: BoolVec,
    healthy_tables: usize,
    dirty_tables: usize,
}

impl Drop for TableFlat {
    fn drop(&mut self) {
        for atom in self.table {
            match atom {
                TableElementKind::NotAllocated => (),
                TableElementKind::Present => (),
                TableElementKind::TableFlat { ptr, .. } => {
                    let _ = unsafe { Box::from_raw(ptr.as_ptr()) };
                }
                TableElementKind::TableBits { ptr, .. } => {
                    let _ = unsafe { Box::from_raw(ptr.as_ptr()) };
                }
            }
        }
    }
}

pub struct TableBits {
    table: BoolVec,
    real_pages: BoolVec,
    atom_size: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct AllocationResult {
    page: PhysPage,
    new_size: usize,
}

pub trait TableImpl: Sized {
    fn empty() -> Self;

    // Returns the free entries of the table
    fn populate_with(
        &mut self,
        el_size: usize,
        start_ptr: PhysAddr<AlignedTo<PAGE_4K>>,
        end_ptr: PhysAddr<AlignedTo<PAGE_4K>>,
    ) -> Result<usize, MemoryError>;

    fn request_page(&mut self, el_size: usize) -> Result<AllocationResult, MemoryError>;
    fn free_page(
        &mut self,
        page: PhysPage,
        el_size: usize,
    ) -> Result<AllocationResult, MemoryError>;

    fn pages_free(&self, el_size: usize) -> Result<usize, MemoryError>;
}

#[derive(Clone)]
pub struct MemoryTable<Table: TableImpl> {
    table: Table,
    element_size: usize,
}

unsafe impl<T: TableImpl> Send for MemoryTable<T> {}
unsafe impl<T: TableImpl> Sync for MemoryTable<T> {}

impl<Table: TableImpl> MemoryTable<Table> {
    pub fn new(element_size: usize) -> Self {
        Self {
            table: Table::empty(),
            element_size,
        }
    }

    pub fn populate_with(
        &mut self,
        start_ptr: PhysAddr<AlignedTo<PAGE_4K>>,
        end_ptr: PhysAddr<AlignedTo<PAGE_4K>>,
    ) -> Result<usize, MemoryError> {
        self.table
            .populate_with(self.element_size, start_ptr, end_ptr)
    }

    #[inline]
    pub fn request_page(&mut self) -> Result<PhysPage, MemoryError> {
        Ok(self.table.request_page(self.element_size)?.page)
    }

    #[inline]
    fn request_page_from_higher(&mut self) -> Result<AllocationResult, MemoryError> {
        self.table.request_page(self.element_size)
    }

    #[inline]
    fn free_page_from_higher(&mut self, page: PhysPage) -> Result<AllocationResult, MemoryError> {
        self.table.free_page(page, self.element_size)
    }

    #[inline]
    pub fn free_page(&mut self, page: PhysPage) -> Result<(), MemoryError> {
        self.table.free_page(page, self.element_size).map(|_| ())
    }

    #[inline]
    pub fn pages_free(&self) -> Result<usize, MemoryError> {
        self.table.pages_free(self.element_size)
    }
}

impl TableImpl for TableFlat {
    fn empty() -> Self {
        Self {
            table: [TableElementKind::NotAllocated; TABLE_SIZE],
            healthy_tables: 0,
            dirty_tables: 0,
            available: BoolVec::new(),
        }
    }

    fn populate_with(
        &mut self,
        el_size: usize,
        start_ptr: PhysAddr<AlignedTo<PAGE_4K>>,
        end_ptr: PhysAddr<AlignedTo<PAGE_4K>>,
    ) -> Result<usize, MemoryError> {
        let el_size_as_ptr: PhysAddr<AlignedTo<PAGE_4K>> = el_size
            .try_into()
            .map_err(|_| MemoryError::NotPageAligned)?;

        // Not 'el_size' population, meaning we must fill these tables now..
        if start_ptr.addr() % el_size != 0 {
            let rel_start = start_ptr.realative_offset(el_size);
            let rel_end = end_ptr.min(el_size_as_ptr);
            let elements = rel_start.distance_to(rel_end) / el_size / TABLE_SIZE;
            let i_element = start_ptr.addr() / el_size;

            self.available.set(i_element, true);

            match self.table[i_element] {
                TableElementKind::Present
                | TableElementKind::TableFlat { .. }
                | TableElementKind::TableBits { .. } => (),
                TableElementKind::NotAllocated if el_size <= LVL1_TABLE => {
                    let atom = &mut self.table[i_element];
                    let bref = Box::leak(Box::new(MemoryTable::new(el_size / TABLE_SIZE)));
                    bref.populate_with(rel_start, rel_end)?;

                    *atom = TableElementKind::TableBits {
                        ptr: bref.into(),
                        atom: elements,
                    };
                    self.dirty_tables += 1;
                }
                TableElementKind::NotAllocated => {
                    let atom = &mut self.table[i_element];
                    let bref = Box::leak(Box::new(MemoryTable::new(el_size / TABLE_SIZE)));
                    bref.populate_with(rel_start, rel_end)?;

                    *atom = TableElementKind::TableFlat {
                        ptr: bref.into(),
                        atom: elements,
                    };
                    self.dirty_tables += 1;
                }
            }
        }

        if end_ptr.addr() % el_size != 0 {
            let rel_start = PhysAddr::try_new(0);
            let rel_end = end_ptr.realative_offset(el_size);
            let elements = rel_start.distance_to(rel_end) / (el_size / TABLE_SIZE);
            let i_element = end_ptr.addr() / el_size;

            self.available.set(i_element, true);

            match self.table[i_element] {
                TableElementKind::Present
                | TableElementKind::TableFlat { .. }
                | TableElementKind::TableBits { .. } => (),
                TableElementKind::NotAllocated if el_size <= LVL1_TABLE => {
                    let atom = &mut self.table[i_element];
                    let bref = Box::leak(Box::new(MemoryTable::new(el_size / TABLE_SIZE)));
                    bref.populate_with(rel_start, rel_end)?;

                    *atom = TableElementKind::TableBits {
                        ptr: bref.into(),
                        atom: elements,
                    };
                    self.dirty_tables += 1;
                }
                TableElementKind::NotAllocated => {
                    let atom = &mut self.table[i_element];
                    let bref = Box::leak(Box::new(MemoryTable::new(el_size / TABLE_SIZE)));
                    bref.populate_with(rel_start, rel_end)?;

                    *atom = TableElementKind::TableFlat {
                        ptr: bref.into(),
                        atom: elements,
                    };
                    self.dirty_tables += 1;
                }
            }
        }

        let atom_start = if start_ptr.addr() % el_size == 0 {
            start_ptr.addr() / el_size
        } else {
            (start_ptr.addr() / el_size) + 1
        };
        let atom_end = end_ptr.addr() / el_size;
        self.healthy_tables += atom_end.saturating_sub(atom_start) as usize;

        for atom_idx in atom_start..atom_end {
            self.available.set(atom_idx as usize, true);
            self.table[atom_idx as usize] = TableElementKind::Present;
        }

        Ok(self.healthy_tables.max(self.dirty_tables.min(1)))
    }

    fn request_page(&mut self, el_size: usize) -> Result<AllocationResult, MemoryError> {
        let el_size_as_ptr: PhysAddr<AlignedTo<PAGE_4K>> = el_size
            .try_into()
            .map_err(|_| MemoryError::NotPageAligned)?;

        if self.healthy_tables == 0 && self.dirty_tables == 0 {
            return Err(MemoryError::OutOfAllocMemory);
        }

        let atom_index = self
            .available
            .find_first_of(true)
            .ok_or(MemoryError::OutOfAllocMemory)?;
        let atom = &mut self.table[atom_index];

        let alloc_result = match atom {
            TableElementKind::Present if el_size <= LVL1_TABLE => {
                let bref = Box::leak(Box::new(MemoryTable::new(el_size / TABLE_SIZE)));
                bref.populate_with(PhysAddr::try_new(0), el_size_as_ptr)?;
                *atom = TableElementKind::TableBits {
                    ptr: bref.into(),
                    atom: TABLE_SIZE,
                };

                self.healthy_tables -= 1;
                self.dirty_tables += 1;

                bref.request_page_from_higher()
            }
            TableElementKind::Present => {
                let bref = Box::leak(Box::new(MemoryTable::new(el_size / TABLE_SIZE)));
                bref.populate_with(PhysAddr::try_new(0), el_size_as_ptr)?;
                *atom = TableElementKind::TableFlat {
                    ptr: bref.into(),
                    atom: TABLE_SIZE,
                };

                self.healthy_tables -= 1;
                self.dirty_tables += 1;

                bref.request_page_from_higher()
            }
            TableElementKind::TableFlat { ptr, .. } => {
                let inner = unsafe { ptr.as_mut() };
                inner.request_page_from_higher()
            }
            TableElementKind::TableBits { ptr, .. } => {
                let inner = unsafe { ptr.as_mut() };
                inner.request_page_from_higher()
            }
            _ => return Err(MemoryError::OutOfAllocMemory),
        }?;

        match atom {
            TableElementKind::TableFlat { atom, .. } | TableElementKind::TableBits { atom, .. } => {
                *atom = alloc_result.new_size;
            }
            _ => unreachable!(),
        }

        if alloc_result.new_size == 0 {
            self.dirty_tables -= 1;
            self.available.set(atom_index, false);
        }

        Ok(AllocationResult {
            page: PhysPage::new(alloc_result.page.page() + ((atom_index * el_size) / PAGE_4K)),
            new_size: self.healthy_tables.max(self.dirty_tables.min(1)),
        })
    }

    fn free_page(
        &mut self,
        page: PhysPage,
        el_size: usize,
    ) -> Result<AllocationResult, MemoryError> {
        let table_index = page.addr().addr() / el_size;
        let inner_index = page.addr().realative_offset(el_size).addr() / PAGE_4K;

        let (previous_size, alloc_result) = match &mut self.table[table_index] {
            TableElementKind::NotAllocated => return Err(MemoryError::NotPhysicalPage),
            TableElementKind::Present => return Err(MemoryError::DoubleFree),
            TableElementKind::TableFlat { ptr, atom } => (
                *atom,
                unsafe { ptr.as_mut() }.free_page_from_higher(PhysPage::new(inner_index))?,
            ),
            TableElementKind::TableBits { ptr, atom } => (
                *atom,
                unsafe { ptr.as_mut() }.free_page_from_higher(PhysPage::new(inner_index))?,
            ),
        };

        match &mut self.table[table_index] {
            TableElementKind::TableFlat { atom, .. } | TableElementKind::TableBits { atom, .. } => {
                *atom = alloc_result.new_size;
            }
            _ => unreachable!(),
        }

        if alloc_result.new_size >= TABLE_SIZE {
            self.healthy_tables += 1;
        } else if previous_size == 0 {
            self.dirty_tables += 1;
        }

        Ok(AllocationResult {
            page,
            new_size: self.healthy_tables.max(self.dirty_tables.min(1)),
        })
    }

    fn pages_free(&self, el_size: usize) -> Result<usize, MemoryError> {
        self.table.iter().try_fold(0, |acc, el| {
            Ok(acc
                + match el {
                    TableElementKind::NotAllocated => 0,
                    TableElementKind::Present => el_size / PAGE_4K,
                    TableElementKind::TableFlat { ptr, .. } => {
                        unsafe { ptr.as_ref() }.pages_free()?
                    }
                    TableElementKind::TableBits { ptr, .. } => {
                        unsafe { ptr.as_ref() }.pages_free()?
                    }
                })
        })
    }
}

impl TableImpl for TableBits {
    fn empty() -> Self {
        Self {
            table: BoolVec::new(),
            atom_size: 0,
            real_pages: BoolVec::new(),
        }
    }

    fn populate_with(
        &mut self,
        el_size: usize,
        start_ptr: PhysAddr<AlignedTo<PAGE_4K>>,
        end_ptr: PhysAddr<AlignedTo<4096>>,
    ) -> Result<usize, MemoryError> {
        if start_ptr >= end_ptr {
            return Err(MemoryError::EntrySizeIsNegative);
        }

        if end_ptr.addr() > (TABLE_SIZE * el_size) {
            return Err(MemoryError::InvalidSize);
        }

        let start = start_ptr.addr() / el_size;
        let end = end_ptr.addr() / el_size;
        self.atom_size += end - start;

        for page in start..end {
            self.table.set(page, true);
            self.real_pages.set(page, true);
        }

        Ok(end - start)
    }

    fn request_page(&mut self, _el_size: usize) -> Result<AllocationResult, MemoryError> {
        if self.atom_size == 0 {
            return Err(MemoryError::OutOfAllocMemory);
        }

        match self.table.find_first_of(true) {
            Some(page_id) if page_id < TABLE_SIZE => {
                self.table.set(page_id, false);
                self.atom_size -= 1;

                Ok(AllocationResult {
                    page: PhysPage::new(page_id),
                    new_size: self.atom_size,
                })
            }
            Some(_) | None => Err(MemoryError::OutOfAllocMemory),
        }
    }

    fn free_page(
        &mut self,
        page: PhysPage,
        _el_size: usize,
    ) -> Result<AllocationResult, MemoryError> {
        if !self.real_pages.get(page.page()) {
            return Err(MemoryError::NotPhysicalPage);
        }

        if self.table.get(page.page()) {
            return Err(MemoryError::DoubleFree);
        }

        self.table.set(page.page(), false);
        self.atom_size += 1;

        Ok(AllocationResult {
            page,
            new_size: self.atom_size,
        })
    }

    fn pages_free(&self, el_size: usize) -> Result<usize, MemoryError> {
        Ok(self.atom_size * (el_size / PAGE_4K))
    }
}

#[cfg(test)]
mod test {

    use crate::page::Page4K;

    use super::*;

    #[test]
    fn test_build_layer_one_table() {
        let mut mt = MemoryTable::<TableBits>::new(LVL0_TABLE);

        // Table relative
        let start: PhysPage<Page4K> = PhysPage::new(3);
        let end: PhysPage<Page4K> = PhysPage::new(10);

        assert_eq!(
            mt.populate_with(start.try_into().unwrap(), end.try_into().unwrap()),
            Ok(7)
        );
        assert_eq!(mt.request_page(), Ok(PhysPage::new(3)));
        assert_eq!(mt.request_page(), Ok(PhysPage::new(4)));
        assert_eq!(mt.request_page(), Ok(PhysPage::new(5)));
        assert_eq!(mt.request_page(), Ok(PhysPage::new(6)));
        assert_eq!(mt.request_page(), Ok(PhysPage::new(7)));
        assert_eq!(mt.request_page(), Ok(PhysPage::new(8)));
        assert_eq!(mt.request_page(), Ok(PhysPage::new(9)));
        assert_eq!(mt.request_page(), Err(MemoryError::OutOfAllocMemory));
    }

    #[test]
    fn test_build_layer_two_table() {
        let mut mt = MemoryTable::<TableFlat>::new(LVL1_TABLE);

        // Table relative
        let start: PhysPage<Page4K> = PhysPage::new(3);
        let end: PhysPage<Page4K> = PhysPage::new(2 * TABLE_SIZE + 12);

        assert_eq!(
            mt.populate_with(start.try_into().unwrap(), end.try_into().unwrap()),
            Ok(1)
        );

        let mut own_pages = BoolVec::new();
        for page in start.page()..end.page() {
            own_pages.set(page as usize, true);
        }

        for _ in start.page()..end.page() {
            let page_id = mt.request_page().unwrap().page();

            assert!(own_pages.get(page_id));
            own_pages.set(page_id, false);
        }
        assert_eq!(mt.request_page(), Err(MemoryError::OutOfAllocMemory));
        assert_eq!(own_pages.find_first_of(true), None);
    }

    #[ignore = "Slow test"]
    #[test]
    fn test_build_layer_three_table() {
        let mut mt = MemoryTable::<TableFlat>::new(LVL2_TABLE);

        // Table relative
        let start: PhysPage<Page4K> = PhysPage::new(TABLE_SIZE + 1);
        let end: PhysPage<Page4K> = PhysPage::new(TABLE_SIZE.pow(2) + TABLE_SIZE * 3 + 12);

        assert_eq!(
            mt.populate_with(start.try_into().unwrap(), end.try_into().unwrap()),
            Ok(3)
        );

        let mut own_pages = BoolVec::new();
        for page in start.page()..end.page() {
            own_pages.set(page as usize, true);
        }

        for _ in start.page()..end.page() {
            let page_id = mt.request_page().unwrap().page();

            assert!(own_pages.get(page_id));
            own_pages.set(page_id, false);
        }
        assert_eq!(mt.request_page(), Err(MemoryError::OutOfAllocMemory));
        assert_eq!(own_pages.find_first_of(true), None);
    }

    #[ignore = "Slow test"]
    #[test]
    fn test_build_layer_four_table() {
        let mut mt = MemoryTable::<TableFlat>::new(LVL3_TABLE);

        // Table relative
        let start: PhysPage<Page4K> = PhysPage::new(TABLE_SIZE + 1);
        let end: PhysPage<Page4K> =
            PhysPage::new(TABLE_SIZE.pow(3) + TABLE_SIZE.pow(2) * 4 + TABLE_SIZE * 3 + 12);

        assert_eq!(
            mt.populate_with(start.try_into().unwrap(), end.try_into().unwrap()),
            Ok(1)
        );

        let mut own_pages = BoolVec::new();
        for page in start.page()..end.page() {
            own_pages.set(page, true);
        }

        for _ in start.page()..end.page() {
            let page_id = mt.request_page().unwrap().page();

            assert!(own_pages.get(page_id));
            own_pages.set(page_id, false);
        }
        assert_eq!(mt.request_page(), Err(MemoryError::OutOfAllocMemory));
        assert_eq!(own_pages.find_first_of(true), None);
    }

    #[test]
    fn test_build_and_free_tables() {
        let mut mt = MemoryTable::<TableFlat>::new(LVL2_TABLE);

        let start: PhysPage<Page4K> = PhysPage::new(0);
        let end: PhysPage<Page4K> = PhysPage::new(TABLE_SIZE * 2 + 16);

        assert_eq!(
            mt.populate_with(start.try_into().unwrap(), end.try_into().unwrap()),
            Ok(1)
        );

        let mut own_pages = BoolVec::new();
        for page in start.page()..end.page() {
            own_pages.set(page, true);
        }

        for _ in start.page()..end.page() {
            let page_id = mt.request_page().unwrap().page();

            assert!(own_pages.get(page_id));
            own_pages.set(page_id, false);
        }
        assert_eq!(mt.request_page(), Err(MemoryError::OutOfAllocMemory));

        for page in start.page()..end.page() {
            mt.free_page(PhysPage::new(page)).unwrap();

            assert!(!own_pages.get(page));
            own_pages.set(page, true);
        }

        for page in start.page()..end.page() {
            assert!(own_pages.get(page));
        }

        assert_eq!(mt.free_page(end), Err(MemoryError::NotPhysicalPage));
    }
}
