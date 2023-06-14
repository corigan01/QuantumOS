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
    pub used: bool
}

impl MemoryDisc {
    pub fn new(ptr: u64, size: u64) -> Self {
        Self {
            ptr,
            size,
            used: false
        }
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

pub struct MemoryBackedAllocator {
    memory_region: UsableRegion,
    allocations: LinkedListComponent<MemoryDisc>
}

impl MemoryBackedAllocator {
    pub fn new(region: UsableRegion) -> Self {
        assert!(region.size() > size_of::<LinkedListComponent<MemoryDisc>>());

        let mut casted_ptr: NonNull<MemoryDisc> = region.ptr().cast();
        let casted_mut = unsafe { casted_ptr.as_mut() };

        *casted_mut = MemoryDisc::new(region.ptr().as_ptr() as u64, region.size() as u64);

        let linked_list_main = LinkedListComponent::new(OwnPtr::from_mut(casted_mut));

        Self {
            memory_region: region,
            allocations: linked_list_main
        }
    }


    pub fn allocate(&mut self, layout: MemoryLayout) -> Result<UsableRegion, AllocErr> {
        let align = layout.alignment();
        let bytes = layout.bytes();

        if bytes == 0 {
            return Err(AllocErr::ImproperConfig);
        }

        for region in self.allocations.iter() {
            if region.used || region.size < bytes as u64 {
                continue;
            }

            let over_head_bytes_needed = region.bytes_to_alignment(align)
                + (size_of::<(MemoryDisc, LinkedListComponent<MemoryDisc>)>() * 2);

            let total_bytes = over_head_bytes_needed + bytes;

            if total_bytes > region.size as usize {
                continue;
            }

            // should have a region that can hold our allocation

            // First lets collect all the PTRs we are going to need to store this allocation.
            // Since this allocation requires splitting one allocation into two, we need to
            // also grab the second ptrs as well
            let typeless_ptr = region.ptr as *mut u8;
            let first_memory_disc_ptr = typeless_ptr as *mut MemoryDisc;
            let first_linked_list_ptr = unsafe { typeless_ptr.add(size_of::<MemoryDisc>()) } as *mut LinkedListComponent<MemoryDisc>;
            let second_memory_disc_ptr = unsafe { typeless_ptr.add(size_of::<(MemoryDisc, LinkedListComponent<MemoryDisc>)>() + bytes)} as *mut MemoryDisc;
            let second_linked_list_ptr = unsafe { typeless_ptr.add(size_of::<(MemoryDisc, LinkedListComponent<MemoryDisc>)>() + size_of::<MemoryDisc>() + bytes) };



        }


        Err(AllocErr::OutOfMemory)
    }

    pub fn free(&mut self, region: UsableRegion) -> Result<(), AllocErr> {
        todo!()
    }
}
