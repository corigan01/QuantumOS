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

use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ptr::NonNull;
use crate::heap::{AllocatorAPI, GlobalAlloc};
use crate::memory_layout::MemoryLayout;


pub struct RawBitmap<Alloc: AllocatorAPI = GlobalAlloc> {
    pub(crate) ptr: NonNull<usize>,
    pub(crate) cap: usize,
    phn: PhantomData<Alloc>
}

impl<Alloc: AllocatorAPI> RawBitmap<Alloc> {
    pub const UNIT_SIZE_BYTES: usize = size_of::<usize>();
    pub const BITS_PER_UNIT: usize = Self::UNIT_SIZE_BYTES * 8;

    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
            phn: PhantomData,
        }
    }

    pub fn read(&self, index: usize) -> usize {
        if index >= self.cap {
            return 0;
        }

        unsafe { *self.ptr.as_ptr().add(index) }
    }

    pub fn write(&mut self, index: usize, value: usize) {
        // We lazy allocate, so if the value is zero, we don't have to allocate because the
        // default is zero!
        if value == 0 {
            return;
        }

        if index >= self.cap {
            self.reserve((index - self.cap) + 1);
        }

        unsafe { *self.ptr.as_ptr().add(index) = value };
    }

    pub fn reserve(&mut self, additional: usize) {
        if additional == 0 {
            return;
        }
        let allocation_disc = MemoryLayout::new(align_of::<usize>(), size_of::<usize>() * (additional + self.cap));

        let allocation = if self.cap == 0 {
            unsafe { Alloc::allocate_zero(allocation_disc) }
                .expect("Unable to reserve Bitmap")
        } else {
            unsafe { Alloc::realloc_zero(self.ptr, allocation_disc) }
                .expect("Unable to reserve Bitmap")
        };

        let new_ptr = NonNull::new(allocation.ptr as *mut usize).unwrap();

        self.ptr = new_ptr;
        self.cap += additional;
    }
}

impl<Alloc: AllocatorAPI> Drop for RawBitmap<Alloc> {
    fn drop(&mut self) {
        unsafe {
            Alloc::free(self.ptr).expect("Unable to drop Bitmap")
        }
    }
}