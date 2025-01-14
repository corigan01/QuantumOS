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

use alloc::{string::String, vec::Vec};

use crate::{MemoryError, pmm::PhysPage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmBackingKind {
    Physical,
    Other(String),
}

pub trait VmBacking {
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
