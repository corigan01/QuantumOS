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

use core::ptr::NonNull;

use alloc::{boxed::Box, vec::Vec};
use boolvec::BoolVec;
use lldebug::logln;

use crate::{MemoryError, phys::PhysMemoryEntry};
extern crate alloc;

pub struct Pmm {
    table: AbstractMemoryTable,
}

const TABLE_SIZE: usize = 256;
const KERNEL_PTR: u64 = 0x100000000000;

const PAGE_SIZE: usize = 4096;

// Lvl0 4 Kib   -- 256 sections
const LVL0_TABLE: u64 = PAGE_SIZE as u64;
// Lvl1 1 Mib   -- 256 sections
const LVL1_TABLE: u64 = LVL0_TABLE * (TABLE_SIZE as u64);
// Lvl2 256 Mib -- 256 sections
const LVL2_TABLE: u64 = LVL1_TABLE * (TABLE_SIZE as u64);
// Lvl3 64 Gib  -- 256 sections
const LVL3_TABLE: u64 = LVL2_TABLE * (TABLE_SIZE as u64);
// Lvl4 16 Tib  -- 256 sections
const LVL4_TABLE: u64 = LVL3_TABLE * (TABLE_SIZE as u64);

trait MemoryTable {
    /// Feeds Memory into the array, calling down each layer until the granularity of the memory
    /// entry is satisfied.
    ///
    /// This process should be lazy, so to avoid causing memory allocations that are not required
    /// to store the data about this memory.
    fn lazy_feed(&mut self, entry: PhysMemoryEntry) -> Result<(), MemoryError>;

    /// If the next table is allocated, get the table pointed to by the index.
    fn entry_at(&self, index: usize) -> Option<&dyn MemoryTable>;

    fn alloc_table_at(&mut self, start_ptr: u64, end_ptr: u64) -> Result<(), MemoryError>;
}

struct AbstractMemoryTable {
    table: [AbstractMemoryEntry; TABLE_SIZE],
    el_size: u64,
}

impl AbstractMemoryTable {
    pub const fn new(el_size: u64, start: u64, end: u64) -> Self {
        Self {
            table: [AbstractMemoryEntry::blank(); TABLE_SIZE],
            el_size,
        }
    }

    const fn addr2index(&self, addr: u64) -> (usize, usize) {
        (
            (addr / self.el_size) as usize,
            (addr % self.el_size) as usize,
        )
    }

    fn is_conjoined(&self, index: usize) -> bool {
        self.table.get(index).is_some_and(|val| {
            self.table
                .get(index + 1)
                .is_some_and(|val_next| val == val_next)
        })
    }

    pub fn populate_with(&mut self, phys_memory: PhysMemoryEntry) -> Result<(), MemoryError> {
        if phys_memory.start & (PAGE_SIZE as u64 - 1) != 0
            || phys_memory.end & (PAGE_SIZE as u64 - 1) != 0
        {
            return Err(MemoryError::NotPageAligned);
        }

        let (mut tbl_start, tbl_start_remainder) = self.addr2index(phys_memory.start);
        let (mut tbl_end, tbl_end_remainder) = self.addr2index(phys_memory.end);

        // If there is a remainder, we need to process it seperately
        if tbl_start_remainder != 0 {
            tbl_start += 1;

            self.alloc_table_at(phys_memory.start, phys_memory.start + self.el_size)?;
        }

        // Same for the end
        if tbl_end_remainder != 0 {
            tbl_end = tbl_end.saturating_sub(1);

            self.alloc_table_at(
                phys_memory.end - (tbl_end_remainder as u64),
                phys_memory.end,
            )?;
        }

        for tbl_idx in tbl_start..=tbl_end {
            self.table[tbl_idx].set_present()?;
        }

        Ok(())
    }
}

//    FREE              PTR
// [........][........................]
//
// If there is no completly free tables, but there are still
// tables with free lower entries it will keep the table's size
// to '1'. If you need a table with an entry larger then the lowest
// entry, then you look for a table with an entry count of >=2.
#[derive(Clone, Copy, PartialEq, Eq)]
struct AbstractMemoryEntry(u64);

impl AbstractMemoryEntry {
    pub const fn blank() -> Self {
        Self(0)
    }

    // The `FREE`
    pub const fn free_count(&self) -> usize {
        (self.0 & 0xFF) as usize
    }

    pub const fn is_present(&self) -> bool {
        (self.0 >> 8) != 0
    }

    pub const fn is_allocated(&self) -> bool {
        (self.0 >> 8) > 1
    }

    pub const fn get_table_ptr(&self) -> Option<NonNull<AbstractMemoryTable>> {
        if self.is_allocated() {
            NonNull::new((self.0 >> 8 | KERNEL_PTR) as *mut _)
        } else {
            None
        }
    }

    pub const fn get_bottom_table_ptr(&self) -> Option<NonNull<BottomTable>> {
        if self.is_allocated() {
            NonNull::new((self.0 >> 8 | KERNEL_PTR) as *mut _)
        } else {
            None
        }
    }

    pub const fn get_conjoined_ptr(&self) -> Option<NonNull<ConjoinedEntry>> {
        if self.is_allocated() {
            NonNull::new((self.0 >> 8 | KERNEL_PTR) as *mut _)
        } else {
            None
        }
    }

    pub fn set_table_ptr(&mut self, ptr: NonNull<AbstractMemoryTable>) -> Result<(), MemoryError> {
        if self.is_allocated() {
            return Err(MemoryError::AlreadyUsed);
        }

        self.remove_present();

        let ptr = ptr.addr().get() as u64;
        self.0 |= (ptr & !(KERNEL_PTR)) << 8;

        Ok(())
    }

    pub fn set_bottom_table_ptr(&mut self, ptr: NonNull<BottomTable>) -> Result<(), MemoryError> {
        if self.is_allocated() {
            return Err(MemoryError::AlreadyUsed);
        }

        self.remove_present();

        let ptr = ptr.addr().get() as u64;
        self.0 |= (ptr & !(KERNEL_PTR)) << 8;

        Ok(())
    }

    pub fn set_conjoined_ptr(&mut self, ptr: NonNull<ConjoinedEntry>) -> Result<(), MemoryError> {
        if self.is_allocated() {
            return Err(MemoryError::AlreadyUsed);
        }

        self.remove_present();

        let ptr = ptr.addr().get() as u64;
        self.0 |= (ptr & !(KERNEL_PTR)) << 8;

        Ok(())
    }

    // DOES NOT DROP THE INNER PTR
    pub const fn zero_entry(&mut self) {
        self.0 = 0;
    }

    pub fn set_present(&mut self) -> Result<(), MemoryError> {
        if self.is_present() {
            return Err(MemoryError::AlreadyUsed);
        }

        self.0 |= 1 << 8;

        Ok(())
    }

    pub const fn remove_present(&mut self) {
        self.0 &= !(1 << 8);
    }
}

// This entry is allocated once, and then the ptr is used in each
// index into the table above where it is taking the memory for.
//
// `S` is the size of *one* entry.
// `NS` is the size of *one* lower entry.
struct ConjoinedEntry {
    start_ptr: u64,
    start_table_leftovers: AbstractMemoryEntry,
    end_ptr: u64,
    end_table_leftovers: AbstractMemoryEntry,
    el_size: u64,
}

impl ConjoinedEntry {
    pub const fn new(el_size: u64, start_ptr: u64, end_ptr: u64) -> Self {
        Self {
            start_ptr,
            start_table_leftovers: AbstractMemoryEntry::blank(),
            end_ptr,
            end_table_leftovers: AbstractMemoryEntry::blank(),
            el_size,
        }
    }
}

// A table that has no lower level tables
//
// These entries are allocated with a bit table
struct BottomTable {
    el_size: u64,
    allocated: BoolVec,
    // The max amount entries that are backed with phys memory
    //
    // This should normally equal the TABLE_SIZE, unless the
    // 'feed' memory was not aligned to an entire table
    page_limit: usize,
    // Conjoined entries
    //
    // 0: table start index
    // 1: table end index
    multi_entries: Vec<(usize, usize)>,
}

impl BottomTable {
    pub const fn new(el_size: u64, start: u64, end: u64) -> Self {
        Self {
            el_size,
            allocated: BoolVec::new(),
            page_limit: 0,
            multi_entries: Vec::new(),
        }
    }
}

impl MemoryTable for AbstractMemoryTable {
    fn lazy_feed(&mut self, entry: PhysMemoryEntry) -> Result<(), MemoryError> {
        todo!()
    }

    fn entry_at(&self, index: usize) -> Option<&dyn MemoryTable> {
        todo!()
    }

    fn alloc_table_at(&mut self, start_ptr: u64, end_ptr: u64) -> Result<(), MemoryError> {
        let (start_table_index, _) = self.addr2index(start_ptr);
        let (end_table_index, _) = self.addr2index(end_ptr);

        match () {
            // Bottom Table
            _ if self.el_size <= LVL1_TABLE => {
                let bref = Box::leak(Box::new(BottomTable::new(
                    self.el_size / TABLE_SIZE as u64,
                    start_ptr,
                    end_ptr,
                )));
                let nnptr = NonNull::new(bref as *mut _).ok_or(MemoryError::PtrWasNull)?;
                self.table[start_table_index].set_bottom_table_ptr(nnptr)?;

                Ok(())
            }
            // Conjoined Entry
            _ if end_table_index - start_table_index > 1 => {
                let bref = Box::leak(Box::new(ConjoinedEntry::new(
                    self.el_size / TABLE_SIZE as u64,
                    start_ptr,
                    end_ptr,
                )));
                let nnptr = NonNull::new(bref as *mut _).ok_or(MemoryError::PtrWasNull)?;
                self.table[start_table_index].set_conjoined_ptr(nnptr)?;

                for i in start_table_index..end_table_index {
                    self.table[i].set_conjoined_ptr(nnptr)?;
                }

                Ok(())
            }
            // Full Table
            _ => {
                let bref = Box::leak(Box::new(AbstractMemoryTable::new(
                    self.el_size / TABLE_SIZE as u64,
                    start_ptr,
                    end_ptr,
                )));
                let nnptr = NonNull::new(bref as *mut _).ok_or(MemoryError::PtrWasNull)?;
                self.table[start_table_index].set_table_ptr(nnptr)?;

                Ok(())
            }
        }
    }
}

impl MemoryTable for BottomTable {
    fn lazy_feed(&mut self, entry: PhysMemoryEntry) -> Result<(), MemoryError> {
        todo!()
    }

    fn entry_at(&self, index: usize) -> Option<&dyn MemoryTable> {
        todo!()
    }

    fn alloc_table_at(&mut self, start_ptr: u64, end_ptr: u64) -> Result<(), MemoryError> {
        todo!()
    }
}

impl MemoryTable for ConjoinedEntry {
    fn lazy_feed(&mut self, entry: PhysMemoryEntry) -> Result<(), MemoryError> {
        todo!()
    }

    fn entry_at(&self, index: usize) -> Option<&dyn MemoryTable> {
        todo!()
    }

    fn alloc_table_at(&mut self, start_ptr: u64, end_ptr: u64) -> Result<(), MemoryError> {
        todo!()
    }
}

#[cfg(test)]
mod test {}
