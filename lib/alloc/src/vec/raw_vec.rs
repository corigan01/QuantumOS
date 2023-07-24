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
use crate::heap::{AllocatorAPI, GlobalAlloc};
use crate::memory_layout::MemoryLayout;

unsafe impl<T: Send, Alloc: AllocatorAPI> Send for RawVec<T, Alloc> {}
unsafe impl<T: Sync, Alloc: AllocatorAPI> Sync for RawVec<T, Alloc> {}

#[allow(dead_code)]
pub struct RawVec<Type, Alloc: AllocatorAPI> {
    pub(crate) ptr: NonNull<Type>,
    pub(crate) cap: usize,
    alloc: Alloc
}

impl<Type> RawVec<Type, GlobalAlloc> {
    pub unsafe fn from_raw_parts(ptr: *mut Type, capacity: usize) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr),
            cap: capacity,
            alloc: GlobalAlloc
        }
    }
}

impl<Type, Alloc: AllocatorAPI> RawVec<Type, Alloc> {
    pub const fn new_in(alloc: Alloc) -> Self {
        let sizeof_type = size_of::<Type>();
        let cap = if sizeof_type == 0 { usize::MAX } else { 0 };

        Self {
            ptr: NonNull::dangling(),
            cap,
            alloc,
        }
    }

    pub fn with_capacity_in(capacity: usize, alloc: Alloc) -> Self {
        let mut new = Self::new_in(alloc);
        new.reserve(capacity);

        new
    }

    pub fn reserve(&mut self, additional: usize) {
        let memory_disc = MemoryLayout::from_type_array::<Type>(self.cap + additional);

        let new_allocation = if self.cap == 0 {
            unsafe { Alloc::allocate(memory_disc) }.expect("Unable to reserve vector!")
        } else {
            unsafe { Alloc::realloc(self.ptr, memory_disc) }.expect("Unable to expand vector!")
        };

        self.ptr = NonNull::new(new_allocation.ptr as *mut Type).unwrap();
        self.cap += additional;
    }
}

impl<Type, Alloc: AllocatorAPI> Drop for RawVec<Type, Alloc> {
    fn drop(&mut self) {
        if self.cap == 0 || size_of::<Type>() == 0 {
            return;
        }

        unsafe {
            Alloc::free(self.ptr).expect("Unable to free vector!");
        }
    }
}