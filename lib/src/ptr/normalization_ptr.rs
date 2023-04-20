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

use crate::ptr::unsafe_ptr::UnsafePtr;

pub struct NormPtr {
    ptr: UnsafePtr<u8>,
    ptr_size_bytes: Option<usize>,
}

impl NormPtr {
    pub fn new(ptr: u64) -> Self {
        Self {
            ptr: UnsafePtr::new(ptr),
            ptr_size_bytes: None,
        }
    }

    pub fn set_ptr_from_u8(&mut self, ptr: *mut u8) {
        self.ptr.set_ptr(ptr);
        self.ptr_size_bytes = Some(1);
    }

    pub fn set_ptr_from_u16(&mut self, ptr: *mut u16) {
        self.ptr.set_ptr(ptr as *mut u8);
        self.ptr_size_bytes = Some(2);
    }

    pub fn set_ptr_from_u32(&mut self, ptr: *mut u32) {
        self.ptr.set_ptr(ptr as *mut u8);
        self.ptr_size_bytes = Some(4);
    }

    pub fn set_ptr_from_u64(&mut self, ptr: *mut u64) {
        self.ptr.set_ptr(ptr as *mut u8);
        self.ptr_size_bytes = Some(8);
    }

    pub fn set_ptr_from_unknown_type(&mut self, ptr: *mut u8, ptr_size_in_bytes: usize) {
        self.ptr.set_ptr(ptr);
        self.ptr_size_bytes = Some(ptr_size_in_bytes);
    }
}
