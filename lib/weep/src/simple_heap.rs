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
use over_stacked::heapless_bits::HeaplessBits;
use crate::AllocErr;

pub enum SimpleAllocationType {
    Free,
    Used
}

pub struct SimpleHeap<const BLOCK_SIZE: usize> {
    map: HeaplessBits<64>,
    ptr: u64,
    size: usize
}

impl<const BLOCK_SIZE: usize> SimpleHeap<BLOCK_SIZE> {
    pub fn new(ptr: *mut u8, size: usize) -> Self {
        Self {
            map: HeaplessBits::new(),
            ptr: ptr as u64,
            size
        }
    }

    pub fn alloc_type<T>(&mut self) -> Result<NonNull<T>, AllocErr> {
        let amount_of_chunks = (size_of::<T>() + (BLOCK_SIZE - 1)) / BLOCK_SIZE;

        let first_free_bits =
            if let Some(value) = self.map.first_of_qty(amount_of_chunks, false) { value }
            else { return Err(AllocErr::OutOfMemory); };

        self.map.set_bit_qty(first_free_bits, amount_of_chunks, true).unwrap();

        let ptr_offset = first_free_bits * BLOCK_SIZE;

        // FIXME: We should know this before we even look for regions
        if ptr_offset > self.size {
            return Err(AllocErr::OutOfMemory);
        }

        let new_ptr = (self.ptr + ptr_offset as u64) as *mut T;

        Ok(unsafe { NonNull::new_unchecked(new_ptr) })
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_allocate_new_type() {

    }

}