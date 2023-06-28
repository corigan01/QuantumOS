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


use core::mem::size_of;
use over_stacked::raw_vec::RawVec;
use crate::usable_region::UsableRegion;

#[derive(Clone, Copy, Debug)]
pub struct HeapEntry {
    ptr: u64,
    size: u64
}

pub struct KernelHeap {
    allocations: RawVec<HeapEntry>
}

impl KernelHeap {
    pub fn new(region: UsableRegion) -> Option<Self> {
        let init_vec_size = (size_of::<HeapEntry>() * 10) + 1;
        if region.size() <= init_vec_size {
            return None;
        }

        let region_start_ptr = region.ptr().cast();
        let mut raw_vec: RawVec<HeapEntry> = RawVec::begin(region_start_ptr, 10);

        let adjusted_ptr = region_start_ptr.as_ptr() as u64 + init_vec_size as u64;
        let adjusted_size = (region.size() - init_vec_size) as u64;

        let init_alloc = HeapEntry {
            ptr: adjusted_ptr,
            size: adjusted_size
        };

        raw_vec.push_within_capacity(init_alloc).unwrap();

        Some(Self {
            allocations: raw_vec
        })
    }
}