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

pub struct UnsafePtr<Type> {
    ptr: u64,
    is_const: bool,
    reserved: PhantomData<Type>,
}

impl<Type> UnsafePtr<Type> {
    pub fn new(ptr: u64) -> Self {
        Self {
            ptr,
            is_const: false,
            reserved: Default::default(),
        }
    }

    pub fn set_const_ptr(&mut self, const_ptr: *const Type) {
        self.ptr = const_ptr as u64;
        self.is_const = true;
    }

    pub fn set_ptr(&mut self, mut_ptr: *mut Type) {
        self.ptr = mut_ptr as u64;
    }

    pub fn get_ptr(&self) -> Option<*const Type> {
        if self.ptr == 0 {
            return None;
        }

        Some(self.ptr as *const Type)
    }

    pub fn get_mut_ptr(&mut self) -> Option<*mut Type> {
        if self.ptr == 0 || !self.is_const {
            return None;
        }

        Some(self.ptr as *mut Type)
    }
}

impl<Type> UnsafePtr<Type>
where
    Type: Copy,
{
    pub unsafe fn collapse_ptr(self) -> Option<Type> {
        if self.ptr == 0 {
            return None;
        }

        Some(*(self.ptr as *const Type))
    }
}
