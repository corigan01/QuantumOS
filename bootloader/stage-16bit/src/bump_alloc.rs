/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
    Part of the Quantum OS Project

Copyright 2024 Gavin Kellam

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

pub struct BumpAlloc {
    current_ptr: *mut u8,
    end: *mut u8,
}

impl BumpAlloc {
    pub unsafe fn new(current_ptr: *mut u8, size: usize) -> Self {
        Self {
            current_ptr,
            end: current_ptr.add(size),
        }
    }

    pub unsafe fn allocate(&mut self, size: usize) -> Option<&'static mut [u8]> {
        let bumped_ptr = self.current_ptr.add(size);
        if bumped_ptr > self.end {
            return None;
        }

        let allocation_start = self.current_ptr;
        self.current_ptr = bumped_ptr;

        Some(core::slice::from_raw_parts_mut(allocation_start, size))
    }

    pub fn push_ptr_to(&mut self, new_ptr: *mut u8) {
        if new_ptr > self.end {
            panic!("Cannot push ptr past end of allocation area!");
        }

        self.current_ptr = new_ptr;
    }

    pub fn align_ptr_to(&mut self, alignment: usize) {
        unsafe {
            self.current_ptr = self
                .current_ptr
                .add(self.current_ptr.align_offset(alignment))
        };
    }
}
