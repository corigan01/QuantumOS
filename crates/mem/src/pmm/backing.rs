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

use super::PhysPage;
use crate::MemoryError;
use alloc::boxed::Box;
use boolvec::BoolVec;
use core::ptr::NonNull;

pub const TABLE_SIZE: usize = 512;

// Single 4 Kib Page
pub const PAGE_SIZE: u64 = 4096;

pub const LVL0_TABLE: u64 = PAGE_SIZE;
pub const LVL1_TABLE: u64 = LVL0_TABLE * (TABLE_SIZE as u64);
pub const LVL2_TABLE: u64 = LVL1_TABLE * (TABLE_SIZE as u64);
pub const LVL3_TABLE: u64 = LVL2_TABLE * (TABLE_SIZE as u64);
pub const LVL4_TABLE: u64 = LVL3_TABLE * (TABLE_SIZE as u64);

pub const OPT_TABLES: [u64; 4] = [LVL1_TABLE, LVL2_TABLE, LVL3_TABLE, LVL4_TABLE];

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
        el_size: u64,
        start_ptr: u64,
        end_ptr: u64,
    ) -> Result<usize, MemoryError>;

    fn request_page(&mut self, el_size: u64) -> Result<AllocationResult, MemoryError>;
    fn free_page(&mut self, page: u64, el_size: u64) -> Result<AllocationResult, MemoryError>;
}

#[derive(Clone)]
pub struct MemoryTable<Table: TableImpl> {
    table: Table,
    element_size: u64,
}

impl<Table: TableImpl> MemoryTable<Table> {
    pub fn new(element_size: u64) -> Self {
        Self {
            table: Table::empty(),
            element_size,
        }
    }

    pub fn populate_with(&mut self, start_ptr: u64, end_ptr: u64) -> Result<usize, MemoryError> {
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
        self.table.free_page(page.0, self.element_size)
    }

    #[inline]
    pub fn free_page(&mut self, page: PhysPage) -> Result<(), MemoryError> {
        self.table.free_page(page.0, self.element_size).map(|_| ())
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
        el_size: u64,
        start_ptr: u64,
        end_ptr: u64,
    ) -> Result<usize, MemoryError> {
        if start_ptr & (PAGE_SIZE - 1) != 0 || end_ptr & (PAGE_SIZE - 1) != 0 {
            return Err(MemoryError::NotPageAligned);
        }

        // Not 'el_size' population, meaning we must fill these tables now..
        if start_ptr % el_size != 0 {
            let rel_start = start_ptr % el_size;
            let rel_end = end_ptr.min(el_size);
            let elements = ((rel_end - rel_start) / (el_size / TABLE_SIZE as u64)) as usize;
            self.available.set((start_ptr / el_size) as usize, true);

            match self.table[(start_ptr / el_size) as usize] {
                TableElementKind::Present
                | TableElementKind::TableFlat { .. }
                | TableElementKind::TableBits { .. } => (),
                TableElementKind::NotAllocated if el_size <= LVL1_TABLE => {
                    let atom = &mut self.table[(start_ptr / el_size) as usize];
                    let bref = Box::leak(Box::new(MemoryTable::new(el_size / TABLE_SIZE as u64)));
                    bref.populate_with(rel_start, rel_end)?;

                    *atom = TableElementKind::TableBits {
                        ptr: bref.into(),
                        atom: elements,
                    };
                    self.dirty_tables += 1;
                }
                TableElementKind::NotAllocated => {
                    let atom = &mut self.table[(start_ptr / el_size) as usize];
                    let bref = Box::leak(Box::new(MemoryTable::new(el_size / TABLE_SIZE as u64)));
                    bref.populate_with(rel_start, rel_end)?;

                    *atom = TableElementKind::TableFlat {
                        ptr: bref.into(),
                        atom: elements,
                    };
                    self.dirty_tables += 1;
                }
            }
        }

        if end_ptr % el_size != 0 {
            let rel_start = 0;
            let rel_end = end_ptr % el_size;
            let elements = ((rel_end - rel_start) / (el_size / TABLE_SIZE as u64)) as usize;
            self.available.set((end_ptr / el_size) as usize, true);

            match self.table[(end_ptr / el_size) as usize] {
                TableElementKind::Present
                | TableElementKind::TableFlat { .. }
                | TableElementKind::TableBits { .. } => (),
                TableElementKind::NotAllocated if el_size <= LVL1_TABLE => {
                    let atom = &mut self.table[(end_ptr / el_size) as usize];
                    let bref = Box::leak(Box::new(MemoryTable::new(el_size / TABLE_SIZE as u64)));
                    bref.populate_with(rel_start, rel_end)?;

                    *atom = TableElementKind::TableBits {
                        ptr: bref.into(),
                        atom: elements,
                    };
                    self.dirty_tables += 1;
                }
                TableElementKind::NotAllocated => {
                    let atom = &mut self.table[(end_ptr / el_size) as usize];
                    let bref = Box::leak(Box::new(MemoryTable::new(el_size / TABLE_SIZE as u64)));
                    bref.populate_with(rel_start, rel_end)?;

                    *atom = TableElementKind::TableFlat {
                        ptr: bref.into(),
                        atom: elements,
                    };
                    self.dirty_tables += 1;
                }
            }
        }

        let atom_start = if start_ptr % el_size == 0 {
            start_ptr / el_size
        } else {
            (start_ptr / el_size) + 1
        };
        let atom_end = end_ptr / el_size;
        self.healthy_tables += (atom_end - atom_start) as usize;

        for atom_idx in atom_start..atom_end {
            self.available.set(atom_idx as usize, true);
            self.table[atom_idx as usize] = TableElementKind::Present;
        }

        Ok(self.healthy_tables.max(self.dirty_tables.min(1)))
    }

    fn request_page(&mut self, el_size: u64) -> Result<AllocationResult, MemoryError> {
        if self.healthy_tables == 0 && self.dirty_tables == 0 {
            return Err(MemoryError::OutOfMemory);
        }

        let atom_index = self
            .available
            .find_first_of(true)
            .ok_or(MemoryError::OutOfMemory)?;
        let atom = &mut self.table[atom_index];

        let alloc_result = match atom {
            TableElementKind::Present if el_size <= LVL1_TABLE => {
                let bref = Box::leak(Box::new(MemoryTable::new(el_size / TABLE_SIZE as u64)));
                bref.populate_with(0, el_size)?;
                *atom = TableElementKind::TableBits {
                    ptr: bref.into(),
                    atom: TABLE_SIZE,
                };

                self.healthy_tables -= 1;
                self.dirty_tables += 1;

                bref.request_page_from_higher()
            }
            TableElementKind::Present => {
                let bref = Box::leak(Box::new(MemoryTable::new(el_size / TABLE_SIZE as u64)));
                bref.populate_with(0, el_size)?;
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
            _ => return Err(MemoryError::OutOfMemory),
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
            page: PhysPage(alloc_result.page.0 + ((atom_index as u64 * el_size) / PAGE_SIZE)),
            new_size: self.healthy_tables.max(self.dirty_tables.min(1)),
        })
    }

    fn free_page(&mut self, page: u64, el_size: u64) -> Result<AllocationResult, MemoryError> {
        let table_index = ((page * PAGE_SIZE) / el_size) as usize;
        let inner_index = ((page * PAGE_SIZE) % el_size) / PAGE_SIZE;

        let (previous_size, alloc_result) = match &mut self.table[table_index] {
            TableElementKind::NotAllocated => return Err(MemoryError::NotPhysicalPage),
            TableElementKind::Present => return Err(MemoryError::DoubleFree),
            TableElementKind::TableFlat { ptr, atom } => (
                *atom,
                unsafe { ptr.as_mut() }.free_page_from_higher(PhysPage(inner_index))?,
            ),
            TableElementKind::TableBits { ptr, atom } => (
                *atom,
                unsafe { ptr.as_mut() }.free_page_from_higher(PhysPage(inner_index))?,
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
            page: PhysPage(page),
            new_size: self.healthy_tables.max(self.dirty_tables.min(1)),
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
        el_size: u64,
        start_ptr: u64,
        end_ptr: u64,
    ) -> Result<usize, MemoryError> {
        if start_ptr & (el_size - 1) != 0 || end_ptr & (el_size - 1) != 0 {
            return Err(MemoryError::NotPageAligned);
        }

        if start_ptr >= end_ptr {
            return Err(MemoryError::EntrySizeIsNegative);
        }

        if end_ptr > (TABLE_SIZE as u64 * el_size) {
            return Err(MemoryError::InvalidSize);
        }

        let start = (start_ptr / el_size) as usize;
        let end = (end_ptr / el_size) as usize;
        self.atom_size += end - start;

        for page in start..end {
            self.table.set(page, true);
            self.real_pages.set(page, true);
        }

        Ok(end - start)
    }

    fn request_page(&mut self, _el_size: u64) -> Result<AllocationResult, MemoryError> {
        if self.atom_size == 0 {
            return Err(MemoryError::OutOfMemory);
        }

        match self.table.find_first_of(true) {
            Some(page_id) if page_id < TABLE_SIZE => {
                self.table.set(page_id, false);
                self.atom_size -= 1;

                Ok(AllocationResult {
                    page: PhysPage(page_id as u64),
                    new_size: self.atom_size,
                })
            }
            Some(_) | None => Err(MemoryError::OutOfMemory),
        }
    }

    fn free_page(&mut self, page: u64, _el_size: u64) -> Result<AllocationResult, MemoryError> {
        if !self.real_pages.get(page as usize) {
            return Err(MemoryError::NotPhysicalPage);
        }

        if self.table.get(page as usize) {
            return Err(MemoryError::DoubleFree);
        }

        self.table.set(page as usize, false);
        self.atom_size += 1;

        Ok(AllocationResult {
            page: PhysPage(page),
            new_size: self.atom_size,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_build_layer_one_table() {
        let mut mt = MemoryTable::<TableBits>::new(LVL0_TABLE);

        // Table relative
        let start = 3;
        let end = 10;

        assert_eq!(mt.populate_with(PAGE_SIZE * start, PAGE_SIZE * end), Ok(7));
        assert_eq!(mt.request_page(), Ok(PhysPage(3)));
        assert_eq!(mt.request_page(), Ok(PhysPage(4)));
        assert_eq!(mt.request_page(), Ok(PhysPage(5)));
        assert_eq!(mt.request_page(), Ok(PhysPage(6)));
        assert_eq!(mt.request_page(), Ok(PhysPage(7)));
        assert_eq!(mt.request_page(), Ok(PhysPage(8)));
        assert_eq!(mt.request_page(), Ok(PhysPage(9)));
        assert_eq!(mt.request_page(), Err(MemoryError::OutOfMemory));
    }

    #[test]
    fn test_build_layer_two_table() {
        let mut mt = MemoryTable::<TableFlat>::new(LVL1_TABLE);

        // Table relative
        let start = 3;
        let end = (TABLE_SIZE as u64 * 2) + 12;

        assert_eq!(mt.populate_with(PAGE_SIZE * start, PAGE_SIZE * end), Ok(1));

        let mut own_pages = BoolVec::new();
        for page in start..end {
            own_pages.set(page as usize, true);
        }

        for _ in start..end {
            let page_id = mt.request_page().unwrap().0 as usize;

            assert!(own_pages.get(page_id));
            own_pages.set(page_id, false);
        }
        assert_eq!(mt.request_page(), Err(MemoryError::OutOfMemory));
        assert_eq!(own_pages.find_first_of(true), None);
    }

    #[ignore = "Slow test"]
    #[test]
    fn test_build_layer_three_table() {
        let mut mt = MemoryTable::<TableFlat>::new(LVL2_TABLE);

        // Table relative
        let start = (TABLE_SIZE as u64) + 1;
        let end = ((TABLE_SIZE * TABLE_SIZE) as u64 * 4) + (TABLE_SIZE as u64 * 3) + 12;

        assert_eq!(mt.populate_with(PAGE_SIZE * start, PAGE_SIZE * end), Ok(3));

        let mut own_pages = BoolVec::new();
        for page in start..end {
            own_pages.set(page as usize, true);
        }

        for _ in start..end {
            let page_id = mt.request_page().unwrap().0 as usize;

            assert!(own_pages.get(page_id));
            own_pages.set(page_id, false);
        }
        assert_eq!(mt.request_page(), Err(MemoryError::OutOfMemory));
        assert_eq!(own_pages.find_first_of(true), None);
    }

    #[ignore = "Slow test"]
    #[test]
    fn test_build_layer_four_table() {
        let mut mt = MemoryTable::<TableFlat>::new(LVL3_TABLE);

        // Table relative
        let start = (TABLE_SIZE as u64) + 1;
        let end = ((TABLE_SIZE * TABLE_SIZE * TABLE_SIZE) as u64 * 1)
            + ((TABLE_SIZE * TABLE_SIZE) as u64 * 4)
            + (TABLE_SIZE as u64 * 3)
            + 12;

        assert_eq!(mt.populate_with(PAGE_SIZE * start, PAGE_SIZE * end), Ok(1));

        let mut own_pages = BoolVec::new();
        for page in start..end {
            own_pages.set(page as usize, true);
        }

        for _ in start as usize..end as usize {
            let page_id = mt.request_page().unwrap().0 as usize;

            assert!(own_pages.get(page_id));
            own_pages.set(page_id, false);
        }
        assert_eq!(mt.request_page(), Err(MemoryError::OutOfMemory));
        assert_eq!(own_pages.find_first_of(true), None);
    }

    #[ignore = "Slow test"]
    #[test]
    fn test_build_layer_five_table() {
        let mut mt = MemoryTable::<TableFlat>::new(LVL4_TABLE);

        // Table relative
        let start = (TABLE_SIZE as u64) + 1;
        let end = (TABLE_SIZE.pow(4) as u64 * 1)
            + ((TABLE_SIZE * TABLE_SIZE * TABLE_SIZE) as u64 * 1)
            + ((TABLE_SIZE * TABLE_SIZE) as u64 * 4)
            + (TABLE_SIZE as u64 * 3)
            + 12;

        assert_eq!(mt.populate_with(PAGE_SIZE * start, PAGE_SIZE * end), Ok(1));

        let mut own_pages = BoolVec::new();
        for page in start..end {
            own_pages.set(page as usize, true);
        }

        for _ in start as usize..end as usize {
            let page_id = mt.request_page().unwrap().0 as usize;

            assert!(own_pages.get(page_id));
            own_pages.set(page_id, false);
        }
        assert_eq!(mt.request_page(), Err(MemoryError::OutOfMemory));
        assert_eq!(own_pages.find_first_of(true), None);
    }

    #[test]
    fn test_build_and_free_tables() {
        let mut mt = MemoryTable::<TableFlat>::new(LVL2_TABLE);

        let start = 0;
        let end = TABLE_SIZE as u64 * 2 + 16;

        assert_eq!(mt.populate_with(PAGE_SIZE * start, PAGE_SIZE * end), Ok(1));

        let mut own_pages = BoolVec::new();
        for page in start..end {
            own_pages.set(page as usize, true);
        }

        for _ in start as usize..end as usize {
            let page_id = mt.request_page().unwrap().0 as usize;

            assert!(own_pages.get(page_id));
            own_pages.set(page_id, false);
        }
        assert_eq!(mt.request_page(), Err(MemoryError::OutOfMemory));

        for page in start..end {
            mt.free_page(PhysPage(page)).unwrap();

            assert!(!own_pages.get(page as usize));
            own_pages.set(page as usize, true);
        }

        for page in start..end {
            assert!(own_pages.get(page as usize));
        }

        assert_eq!(
            mt.free_page(PhysPage(end)),
            Err(MemoryError::NotPhysicalPage)
        );
    }
}
