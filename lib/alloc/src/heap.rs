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
use core::ptr::NonNull;
use over_stacked::linked_list::LinkedListComponent;
use quantum_utils::own_ptr::OwnPtr;
use crate::AllocErr;
use crate::memory_layout::MemoryLayout;
use crate::usable_region::UsableRegion;


#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct MemoryDisc {
    pub ptr: u64,
    pub size: u64,
    pub used: bool,
    pub next: u64
}

impl MemoryDisc {
    pub fn new(ptr: *mut u8, size: usize) -> Self {
        Self {
            ptr: ptr as u64,
            size: size as u64,
            used: false,
            next: 0
        }
    }

    pub fn add_next(&mut self, next: NonNull<Self>) {
        self.next = next.as_ptr() as u64;
    }

    pub fn next_ptr(&self) -> Option<NonNull<Self>> {
        NonNull::new(self.next as *mut Self)
    }

    pub fn ptr_alignment(&self) -> usize {
        let ptr = self.ptr as usize;
        assert_ne!(ptr, 0);

        let mut max_alignment_found = 0;
        loop {
            if ptr & (1 << max_alignment_found) > 0 {
                break max_alignment_found;
            }

            max_alignment_found += 1;
        }
    }

    pub fn bytes_to_alignment(&self, alignment: usize) -> usize {
        let current_alignment = self.ptr_alignment();

        if current_alignment >= alignment {
            return 0;
        }

        alignment - current_alignment
    }
}

pub struct LinkedListMemoryAllocator {
    memory_regions: NonNull<MemoryDisc>
}

impl LinkedListMemoryAllocator {
    pub fn new(region: UsableRegion) -> Self {
        let alloc_ptr: NonNull<MemoryDisc> = region.ptr().cast();
        unsafe { *alloc_ptr.as_mut() = MemoryDisc::new(region.ptr().as_ptr(), region.size()) };

        Self {
            memory_regions: alloc_ptr
        }
    }

    fn run_on_all_disc<Function>(&self, function: Function) -> Option<MemoryDisc>
        where Function: FnMut(MemoryDisc) -> bool {

        let mut looping_disc = unsafe { *self.memory_regions.as_ptr() };

        loop {
            let is_returnable = function(looping_disc);

            if is_returnable {
                return Some(looping_disc);
            }

            let Some(maybe_next) = looping_disc.next_ptr() else {
                return None;
            };

            looping_disc = unsafe { *maybe_next.unwrap().as_ptr() };
        }
    }

    pub fn alloc(&mut self, layout: MemoryLayout) -> Result<UsableRegion, AllocErr> {
        let align = layout.alignment();
        let bytes = layout.bytes() as u64;
        let bytes_for_disc = size_of::<MemoryDisc>();

        let Some(working_disc) = self.run_on_all_disc(|disc| {
            if disc.used || bytes > disc.size {
                return false;
            }

            let overhead_bytes = bytes_for_disc
                + disc.bytes_to_alignment(align);

            let total_bytes = overhead_bytes as u64 + bytes;

            if disc.size >= total_bytes {
                true;
            }

            false
        }) else { return Err(AllocErr::OutOfMemory); };

        let bytes_to_align = working_disc.bytes_to_alignment(align);

        let pushed_ptr = working_disc.ptr + bytes_to_align;
        let new_size = working_disc.size - (bytes_to_align + bytes_for_disc);

        if new_size > (bytes_for_disc + bytes) as u64 {
            // We need to split into two since we have bytes left over
        }

        // We need to change the allocation to used

        todo!()
    }

    pub fn free(&mut self, region: UsableRegion) -> Result<(), AllocErr> {
        todo!()
    }

}