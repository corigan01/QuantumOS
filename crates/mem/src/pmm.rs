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

use alloc::boxed::Box;

use crate::{
    MemoryError,
    phys::{PhysMemoryKind, PhysMemoryMap},
};

extern crate alloc;

mod backing;

// PAGE ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysPage(pub u64);

pub struct Pmm {
    table: Box<backing::MemoryTable<backing::TableFlat>>,
}

impl Pmm {
    pub fn new<const SIZE: usize>(memory_map: &PhysMemoryMap<SIZE>) -> Result<Self, MemoryError> {
        let total_real_memory = memory_map.sdram_bytes() as u64;

        let mut opt_table = *backing::OPT_TABLES.last().unwrap();
        for table_size in backing::OPT_TABLES {
            if total_real_memory < (table_size * backing::TABLE_SIZE as u64) {
                opt_table = table_size;
                break;
            }
        }

        let mut table = Box::new(backing::MemoryTable::new(opt_table));

        memory_map
            .iter()
            .filter(|entry| {
                entry.kind == PhysMemoryKind::Free && entry.start >= (1 * util::consts::MIB as u64)
            })
            .try_for_each(|entry| {
                let (aligned_start, aligned_end) =
                    util::align_range_to(entry.start, entry.end, 4096);
                table.populate_with(aligned_start, aligned_end).map(|_| ())
            })?;

        Ok(Self { table })
    }

    pub fn allocate_page(&mut self) -> Result<PhysPage, MemoryError> {
        self.table.request_page()
    }

    pub fn free_page(&mut self, page: PhysPage) -> Result<(), MemoryError> {
        self.table.free_page(page)
    }
}
