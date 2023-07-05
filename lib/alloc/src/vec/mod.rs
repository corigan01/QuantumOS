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

use core::ptr;
use core::slice::{Iter, IterMut};
use crate::heap::{AllocatorAPI, GlobalAlloc};
use crate::vec::raw_vec::RawVec;

mod raw_vec;

pub struct Vec<Type, Alloc: AllocatorAPI = GlobalAlloc>{
    raw: RawVec<Type, Alloc>,
    len: usize
}

impl<Type, Alloc: AllocatorAPI> Vec<Type, Alloc> {
    pub const fn new() -> Self {
        Self {
            raw: RawVec::new(),
            len: 0
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut tmp = Self::new();
        tmp.reserve(capacity);

        tmp
    }

    pub const fn as_ptr(&self) -> *const Type {
        self.raw.ptr.as_ptr() as *const Type
    }

    pub fn as_mut_ptr(&mut self) -> *mut Type {
        self.raw.ptr.as_ptr()
    }

    pub fn as_slice<'a>(&self) -> &'a [Type] {
        unsafe {
            core::slice::from_raw_parts(
                self.raw.ptr.as_ptr(),
                self.len
            )
        }
    }

    pub fn as_mut_slice<'a>(&self) -> &'a mut [Type] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.raw.ptr.as_ptr(),
                self.len
            )
        }
    }

    pub fn capacity(&self) -> usize {
        self.raw.cap
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn reserve(&mut self, additional: usize) {
        self.raw.reserve(additional);
    }

    pub fn remaining(&self) -> usize {
        self.raw.cap - self.len
    }

    fn should_reserve(&self) -> bool {
        self.remaining() == 0
    }

    pub fn push(&mut self, value: Type) {
        if self.should_reserve() {
            self.reserve(1);
        }

        unsafe { ptr::write(self.as_mut_ptr().add(self.len), value) };

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<Type> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        Some(unsafe {
            ptr::read(self.as_ptr().add(self.len))
        })
    }

    pub fn insert(&mut self, index: usize, value: Type) {
        assert!(index <= self.len,
                "Index out of bounds! Got index {}, but len is only {}", index, self.len);

        if self.should_reserve() {
            self.reserve(1);
        }

        unsafe {
            let ptr = self.as_mut_ptr().add(index);

            if index < self.len {
                ptr::copy(ptr, ptr.add(1), self.len - index);
            }

            ptr::write(ptr, value);
        }

        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> Type {
        assert!(index <= self.len,
                "Index out of bounds! Got index {}, but len is only {}", index, self.len);


        let return_value;
        unsafe {
            let ptr = self.as_mut_ptr().add(index);

            return_value = ptr::read(ptr);

            ptr::copy(ptr.add(1), ptr, self.len - index - 1);
        }

        self.len -= 1;
        return_value
    }

    pub fn iter(&self) -> Iter<Type> {
        self.as_slice().iter()
    }

    pub fn mut_iter(&mut self) -> IterMut<Type> {
        self.as_mut_slice().iter_mut()
    }
}

impl<Type, Alloc: AllocatorAPI> Drop for Vec<Type, Alloc> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
    }
}

#[cfg(test)]
mod test {
    use crate::heap::set_example_allocator;
    use super::*;

    #[test]
    fn test_push_of_elements() {
        set_example_allocator(4096);

        let mut new_vector: Vec<i32> = Vec::new();

        new_vector.push(0);
        new_vector.push(1);
        new_vector.push(2);
        new_vector.push(3);

        assert_eq!(new_vector.as_slice(), &[0, 1, 2, 3]);
    }

    #[test]
    fn test_push_a_bunch_of_elements() {
        set_example_allocator(4096);
        let mut vector: Vec<i32> = Vec::new();

        for i in 0..10 {
            vector.push(i);
        }


    }


}