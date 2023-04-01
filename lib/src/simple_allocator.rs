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

pub struct SimpleAllocator<'a> {
    memory_slice: &'a mut [u8],
    used_memory: usize,
}

impl<'a> SimpleAllocator<'a> {
    fn test_allocation(&mut self) -> bool {
        if let Some(region) = self.allocate_region(1) {
            let test_byte = 0xde;

            region[0] = test_byte;
            region[0] += 1;

            region[0] == test_byte + 1
        } else {
            false
        }
    }

    pub fn new(memory_region: &'a mut [u8]) -> Self {
        Self {
            memory_slice: memory_region,
            used_memory: 0,
        }
    }

    pub unsafe fn new_from_ptr(ptr: *mut u8, size: usize) -> Option<Self> {
        let mut allocator = Self {
            memory_slice: core::slice::from_raw_parts_mut(ptr, size),
            used_memory: 0,
        };

        if !allocator.test_allocation() {
            return None;
        }

        return Some(allocator);
    }

    pub fn allocate_region(&mut self, size: usize) -> Option<&'a mut [u8]> {
        if self.used_memory + size > self.memory_slice.len() {
            return None;
        }

        let slice = unsafe {
            core::slice::from_raw_parts_mut(
                self.memory_slice.as_mut_ptr().add(self.used_memory),
                size,
            )
        };

        self.used_memory += size;

        Some(slice)
    }
}
