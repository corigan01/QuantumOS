/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
  Part of the Quantum OS Project

Copyright 2023 Gavin Kellam

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

#![no_std]
#![feature(error_in_core)]
#![feature(exclusive_range_pattern)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::error::FsError;

mod abstract_buffer;
pub mod disks;
pub mod error;
pub mod filesystems;
pub mod io;
mod node;

mod fs;
pub use crate::fs::*;

mod file;
pub use crate::file::*;

pub type FsResult<T> = Result<T, FsError>;

#[cfg(test)]
pub fn set_example_allocator(size_in_bytes: usize) {
    use qk_alloc::heap::alloc::KernelHeap;
    use qk_alloc::heap::{free_lock, reserve_lock, set_global_alloc, THE_GLOBAL_ALLOC};
    use qk_alloc::usable_region::UsableRegion;

    unsafe {
        if THE_GLOBAL_ALLOC.is_some() {
            return;
        }
    }

    reserve_lock();

    extern crate alloc;

    let memory_layout = core::alloc::Layout::from_size_align(size_in_bytes, 1).unwrap();
    unsafe {
        let allocation = alloc::alloc::alloc(memory_layout);

        let usable_region = UsableRegion::from_raw_parts(allocation, size_in_bytes).unwrap();
        let new_kern_heap = KernelHeap::new(usable_region).unwrap();

        set_global_alloc(new_kern_heap);
    }

    free_lock();
}
