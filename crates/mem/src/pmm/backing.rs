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
use lldebug::logln;

pub const TABLE_SIZE: usize = 256;

// Single 4 Kib Page
const PAGE_SIZE: u64 = 4096;

pub const LVL0_TABLE: u64 = PAGE_SIZE;
pub const LVL1_TABLE: u64 = LVL0_TABLE * (TABLE_SIZE as u64);
pub const LVL2_TABLE: u64 = LVL1_TABLE * (TABLE_SIZE as u64);
pub const LVL3_TABLE: u64 = LVL2_TABLE * (TABLE_SIZE as u64);
pub const LVL4_TABLE: u64 = LVL3_TABLE * (TABLE_SIZE as u64);

#[derive(Clone, Copy)]
pub struct MemoryAtom(u64, usize);

impl MemoryAtom {
    pub const fn empty() -> Self {
        Self(0, 0)
    }

    pub const fn set_size(&mut self, size: usize) {
        self.1 = size;
    }

    pub const fn size(&self) -> usize {
        self.1
    }

    pub fn put_ptr<T: TableImpl>(&mut self, ptr: NonNull<MemoryTable<T>>) {
        self.0 = ptr.addr().get() as u64;
    }

    pub const fn set_present(&mut self) {
        if !self.is_present() {
            self.0 |= 1;
        }
    }

    pub const fn is_present(&self) -> bool {
        self.0 != 0
    }

    pub const fn is_allocated(&self) -> bool {
        self.0 > 1
    }

    fn get_ptr<T>(&self) -> Option<NonNull<T>> {
        NonNull::new((self.0) as *mut _)
    }

    pub fn get<T: TableImpl>(&self) -> Option<&MemoryTable<T>> {
        if self.is_allocated() {
            Some(unsafe { &*(self.get_ptr()?.as_ptr()) })
        } else {
            None
        }
    }

    pub fn get_mut<T: TableImpl>(&mut self) -> Option<&mut MemoryTable<T>> {
        if self.is_allocated() {
            Some(unsafe { &mut *(self.get_ptr()?.as_ptr()) })
        } else {
            None
        }
    }
}

struct TableFlat {
    table: [MemoryAtom; TABLE_SIZE],
    healthy_tables: usize,
    dirty_tables: usize,
}

struct TableBits {
    table: BoolVec,
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
    fn free_page(&mut self, page: PhysPage) -> Result<(), MemoryError>;
}

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

    pub fn free_page(&mut self, page: PhysPage) -> Result<(), MemoryError> {
        self.table.free_page(page)
    }
}

impl TableImpl for TableFlat {
    fn empty() -> Self {
        Self {
            table: [MemoryAtom::empty(); TABLE_SIZE],
            healthy_tables: 0,
            dirty_tables: 0,
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

            if !self.table[(start_ptr / el_size) as usize].is_present() {
                let atom = &mut self.table[(start_ptr / el_size) as usize];
                if el_size <= LVL1_TABLE {
                    let bref = Box::leak(Box::new(MemoryTable::<TableBits>::new(
                        el_size / TABLE_SIZE as u64,
                    )));
                    bref.populate_with(rel_start, rel_end)?;
                    atom.put_ptr(bref.into());
                    atom.set_size(elements);
                } else {
                    let bref = Box::leak(Box::new(MemoryTable::<TableFlat>::new(
                        el_size / TABLE_SIZE as u64,
                    )));
                    bref.populate_with(rel_start, rel_end)?;
                    atom.put_ptr(bref.into());
                    atom.set_size(elements);
                }

                self.dirty_tables += 1;
            }
        }

        if end_ptr % el_size != 0 {
            let rel_start = 0;
            let rel_end = end_ptr % el_size;
            let elements = ((rel_end - rel_start) / (el_size / TABLE_SIZE as u64)) as usize;

            if !self.table[(end_ptr / el_size) as usize].is_present() {
                let atom = &mut self.table[(end_ptr / el_size) as usize];
                if el_size <= LVL1_TABLE {
                    let bref = Box::leak(Box::new(MemoryTable::<TableBits>::new(
                        el_size / TABLE_SIZE as u64,
                    )));
                    bref.populate_with(rel_start, rel_end)?;
                    atom.put_ptr(bref.into());
                    atom.set_size(elements);
                } else {
                    let bref = Box::leak(Box::new(MemoryTable::<TableFlat>::new(
                        el_size / TABLE_SIZE as u64,
                    )));
                    bref.populate_with(rel_start, rel_end)?;
                    atom.put_ptr(bref.into());
                    atom.set_size(elements);
                }

                self.dirty_tables += 1;
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
            let atom = &mut self.table[atom_idx as usize];
            atom.set_present();
            atom.set_size(TABLE_SIZE);
        }

        Ok(self.healthy_tables.max(self.dirty_tables.min(1)))
    }

    fn request_page(&mut self, el_size: u64) -> Result<AllocationResult, MemoryError> {
        if self.healthy_tables == 0 && self.dirty_tables == 0 {
            return Err(MemoryError::OutOfMemory);
        }

        let optimal_index = self
            .table
            .iter()
            .enumerate()
            .filter(|(_, atom)| atom.is_present() && atom.size() > 0)
            .min_by(|(_, lhs), (_, rhs)| lhs.size().cmp(&rhs.size()))
            .map(|(i, _)| i)
            .ok_or(MemoryError::OutOfMemory)?;

        let atom = &mut self.table[optimal_index];
        let page = if atom.is_allocated() {
            let AllocationResult { page, new_size } = if el_size <= LVL1_TABLE {
                atom.get_mut::<TableBits>()
                    .unwrap()
                    .request_page_from_higher()
            } else {
                atom.get_mut::<TableFlat>()
                    .unwrap()
                    .request_page_from_higher()
            }?;

            if new_size == 0 {
                self.dirty_tables -= 1;
            }
            atom.set_size(new_size);

            page
        } else {
            let AllocationResult { page, .. } = if el_size <= LVL1_TABLE {
                let bref = Box::leak(Box::new(MemoryTable::<TableBits>::new(
                    el_size / TABLE_SIZE as u64,
                )));
                bref.populate_with(0, el_size)?;
                atom.put_ptr(bref.into());
                atom.set_size(TABLE_SIZE);

                bref.request_page_from_higher()
            } else {
                let bref = Box::leak(Box::new(MemoryTable::<TableFlat>::new(
                    el_size / TABLE_SIZE as u64,
                )));
                bref.populate_with(0, el_size)?;
                atom.put_ptr(bref.into());
                atom.set_size(TABLE_SIZE);

                bref.request_page_from_higher()
            }?;

            // Since we downgraded a table, we remove one from our healthy table list
            self.healthy_tables -= 1;
            self.dirty_tables += 1;

            page
        };

        Ok(AllocationResult {
            page: PhysPage(page.0 + ((optimal_index as u64 * el_size) / PAGE_SIZE)),
            new_size: self.healthy_tables.max(self.dirty_tables.min(1)),
        })
    }

    fn free_page(&mut self, page: PhysPage) -> Result<(), MemoryError> {
        core::todo!()
    }
}

impl TableImpl for TableBits {
    fn empty() -> Self {
        Self {
            table: BoolVec::new(),
            atom_size: 0,
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

    fn free_page(&mut self, page: PhysPage) -> Result<(), MemoryError> {
        core::todo!()
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

    #[test]
    fn test_build_layer_three_table() {
        lldebug::testing_stdout!();
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
}
