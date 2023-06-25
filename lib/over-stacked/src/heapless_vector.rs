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

use core::fmt::{Debug, Formatter};
use core::mem;
use core::ptr::NonNull;
use core::slice::{Iter, IterMut};
use crate::raw_vec::{RawVec, RawVecErr};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HeaplessVecErr {
    VectorFull,
    OutOfBounds,
    Invalid
}

impl Into<HeaplessVecErr> for RawVecErr {
    fn into(self) -> HeaplessVecErr {
        match self {
            RawVecErr::Invalid => HeaplessVecErr::Invalid,
            RawVecErr::OutOfBounds => HeaplessVecErr::OutOfBounds,
            RawVecErr::NotEnoughMem => HeaplessVecErr::VectorFull
        }
    }
}

#[allow(dead_code)]
pub struct HeaplessVec<Type, const SIZE: usize> {
    internal_data: [Type; SIZE],
    raw_vec: RawVec<Type>
}

unsafe impl<Type, const SIZE: usize> Send for HeaplessVec<Type, SIZE> {}
unsafe impl<Type, const SIZE: usize> Sync for HeaplessVec<Type, SIZE> {}

impl<Type, const SIZE: usize> Clone for HeaplessVec<Type, SIZE>
where
    Type: Clone,
{
    fn clone(&self) -> Self {
        let mut new_vector = HeaplessVec::new();

        for value in self.raw_vec.iter() {
            new_vector.push_within_capacity(value.clone()).unwrap();
        }

        new_vector
    }
}

impl<Type, const SIZE: usize> HeaplessVec<Type, SIZE> {
    pub fn new() -> Self {
        // FIXME: For some reason the `cell`'s ptr that its assigned here, is not
        // its ptr forever! This is a huge problem because we want the ptr to be the same
        // the entire structs lifetime. This is causing our RawVec ptr to be
        // dangling and then cause undefined behavior.

        let cell: [Type; SIZE] = unsafe { mem::zeroed() };

        let mut self_data = Self {
            internal_data: cell,
            raw_vec: RawVec::new()
        };

        self_data.raw_vec.grow(NonNull::new(self_data.internal_data.as_mut_ptr()).unwrap(),
                              SIZE).unwrap();
        self_data
    }


    pub fn len(&self) -> usize {
        self.raw_vec.len()
    }

    pub fn capacity(&self) -> usize {
        SIZE
    }

    pub fn push_within_capacity(&mut self, value: Type) -> Result<(), HeaplessVecErr> {
        unsafe { self.raw_vec.without_checking_set_new_ptr(self.internal_data.as_mut_ptr()) };
        self.raw_vec.push_within_capacity(value).map_err(|e| e.into())
    }

    pub fn pop(&mut self) -> Option<Type> {
        unsafe { self.raw_vec.without_checking_set_new_ptr(self.internal_data.as_mut_ptr()) };
        self.raw_vec.pop()
    }

    pub fn remove(&mut self, index: usize) -> Option<Type> {
        unsafe { self.raw_vec.without_checking_set_new_ptr(self.internal_data.as_mut_ptr()) };
        self.raw_vec.remove(index)
    }

    pub fn push_vec<const RHS_SIZE: usize>(
        &mut self,
        rhs: HeaplessVec<Type, RHS_SIZE>,
    ) -> Result<(), HeaplessVecErr> {
        unsafe { self.raw_vec.without_checking_set_new_ptr(self.internal_data.as_mut_ptr()) };
        self.raw_vec.push_vec(rhs.raw_vec).map_err(|e| e.into())
    }

    pub fn retain<Function>(&mut self, runner: Function)
    where
        Function: FnMut(&Type) -> bool,
    {
        unsafe { self.raw_vec.without_checking_set_new_ptr(self.internal_data.as_mut_ptr()) };
        self.raw_vec.retain(runner);
    }

    pub fn insert(&mut self, index: usize, element: Type) -> Result<(), HeaplessVecErr> {
        unsafe { self.raw_vec.without_checking_set_new_ptr(self.internal_data.as_mut_ptr()) };
        self.raw_vec.insert(index, element).map_err(|e| e.into())
    }

    pub fn find_index_of(&self, value: &Type) -> Option<usize>
    where
        Type: PartialEq,
    {
        self.raw_vec.find_index_of(value)
    }

    pub fn get(&self, index: usize) -> Option<&Type> {
        self.raw_vec.get_ref(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Type> {
        unsafe { self.raw_vec.without_checking_set_new_ptr(self.internal_data.as_mut_ptr()) };
        self.raw_vec.get_mut(index)
    }

    pub fn as_slice(&self) -> &[Type] {
        self.raw_vec.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [Type] {
        unsafe { self.raw_vec.without_checking_set_new_ptr(self.internal_data.as_mut_ptr()) };
        self.raw_vec.as_mut_slice()
    }

    pub fn as_ptr(&self) -> *const Type {
        self.raw_vec.as_ptr()
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut Type {
        unsafe { self.raw_vec.without_checking_set_new_ptr(self.internal_data.as_mut_ptr()) };
        self.raw_vec.as_mut()
    }

    pub fn iter(&self) -> Iter<Type> {
        self.as_slice().iter()
    }

    pub fn mut_iter(&mut self) -> IterMut<Type> {
        unsafe { self.raw_vec.without_checking_set_new_ptr(self.internal_data.as_mut_ptr()) };
        self.as_mut_slice().iter_mut()
    }
}

impl<Type, const SIZE: usize> Debug for HeaplessVec<Type, SIZE>
where
    Type: Sized,
    Type: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.raw_vec)
        } else {
            write!(f, "{:?}", self.raw_vec)
        }
    }
}

impl<Type, const SIZE: usize> Default for HeaplessVec<Type, SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_super_small_vector() {
        let mut vec: HeaplessVec<u8, 1> = HeaplessVec::new();

        assert_eq!(vec.push_within_capacity(0), Ok(()));
        assert_eq!(vec.push_within_capacity(0), Err(HeaplessVecErr::VectorFull));
    }

    #[test]
    fn new_vector_test() {
        let mut new_vector: HeaplessVec<i32, 10> = HeaplessVec::new();

        assert_eq!(new_vector.capacity(), 10);
        assert_eq!(new_vector.len(), 0);

        new_vector.push_within_capacity(120).unwrap();

        assert_eq!(new_vector.capacity(), 10);
        assert_eq!(new_vector.len(), 1);

        assert_eq!(new_vector.iter().next(), Some(&120));
    }

    #[test]
    fn remove_element_from_vector() {
        let mut vec: HeaplessVec<i32, 10> = HeaplessVec::new();

        for i in 0..10 {
            vec.push_within_capacity(i).unwrap();
        }

        assert_eq!(vec.len(), 10);
        assert_eq!(vec.capacity(), 10);
        assert_eq!(vec.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        vec.remove(0).unwrap();

        assert_eq!(vec.len(), 9);
        assert_eq!(vec.as_slice(), &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn insert_element_into_vector() {
        let mut vec: HeaplessVec<i32, 10> = HeaplessVec::new();

        assert_eq!(vec.len(), 0);

        vec.insert(0, 69).unwrap();

        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(0), Some(&69));

        assert_eq!(vec.insert(10, 320), Err(HeaplessVecErr::OutOfBounds));
    }

    #[test]
    fn test_push_vec() {
        let mut total: HeaplessVec<i32, 20> = HeaplessVec::new();

        total.push_within_capacity(10).unwrap();

        let mut new: HeaplessVec<i32, 10> = HeaplessVec::new();
        for i in 0..10 {
            new.push_within_capacity(i).unwrap();
        }

        total.push_vec(new).unwrap();

        assert_eq!(total.len(), 11);

        assert_eq!(total.get(0), Some(&10));
        for i in 1..11 {
            assert_eq!(*total.get(i as usize).unwrap(), i - 1);
        }
    }

}
