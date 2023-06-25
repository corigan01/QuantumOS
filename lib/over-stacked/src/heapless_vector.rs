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
use core::mem::MaybeUninit;
use core::slice::{Iter, IterMut};
use crate::raw_vec::RawVec;


#[derive(Copy)]
pub struct HeaplessVec<'a, Type, const SIZE: usize> {
    internal_data: [MaybeUninit<Type>; SIZE],
    raw_vec: RawVec<'a, Type>

}

impl<'a, Type, const SIZE: usize> Clone for HeaplessVec<'a, Type, SIZE>
where
    Type: Clone,
{
    fn clone(&self) -> Self {
        let mut new_vector = HeaplessVec::new();

        for index in 0..self.used_data {
            let copied_value = unsafe { self.internal_data[index].assume_init_read() };

            new_vector.push_within_capacity(copied_value).unwrap();
        }

        new_vector
    }
}

impl<'a, Type, const SIZE: usize> HeaplessVec<'a, Type, SIZE> {

    pub fn new() -> Self {
        Self {
            internal_data: unsafe { mem::zeroed() },
            raw_vec,
        }
    }


    pub const fn len(&self) -> usize {
        self.used_data
    }

    pub const fn capacity(&self) -> usize {
        SIZE
    }

    pub fn push_within_capacity(&mut self, value: Type) -> Result<(), HeaplessVecErr> {
        if self.used_data >= SIZE {
            return Err(HeaplessVecErr::VectorFull);
        }

        let index = self.used_data;
        self.internal_data[index].write(value);

        self.used_data += 1;

        Ok(())
    }

    pub fn pop(&mut self) -> Option<Type> {
        if self.used_data == 0 {
            return None;
        }

        self.used_data -= 1;

        Some(unsafe { self.internal_data[self.used_data + 1].assume_init_read() })
    }

    pub fn remove(&mut self, index: usize) -> Option<Type> {
        if self.used_data == 0 && index > self.used_data {
            return None;
        }

        let moved_out_data = unsafe { self.internal_data[index].assume_init_read() };

        for i in index..(self.used_data - 1) {
            self.internal_data.swap(i, i + 1);
        }

        self.used_data -= 1;

        Some(moved_out_data)
    }

    pub fn push_vec<const RHS_SIZE: usize>(
        &mut self,
        rhs: HeaplessVec<Type, RHS_SIZE>,
    ) -> Result<(), HeaplessVecErr> {
        for rhs_index in 0..rhs.used_data {
            unsafe {
                self.push_within_capacity(rhs.internal_data[rhs_index].assume_init_read())?;
            }
        }

        Ok(())
    }

    pub fn retain<Function>(&mut self, mut runner: Function)
    where
        Function: FnMut(&Type) -> bool,
    {
        let mut removed_indexes: HeaplessVec<bool, SIZE> = HeaplessVec::new();

        for value in self.iter() {
            let run_result = runner(value);

            removed_indexes.push_within_capacity(run_result).unwrap();
        }

        let mut deletion_offset = 0;
        for (index, should_keep) in removed_indexes.iter().enumerate() {
            if !should_keep {
                self.remove(index - deletion_offset);
                deletion_offset += 1;
            }
        }
    }

    pub fn insert(&mut self, index: usize, element: Type) -> Result<(), HeaplessVecErr> {
        if self.used_data >= SIZE {
            return Err(HeaplessVecErr::VectorFull);
        }
        if index > self.used_data {
            return Err(HeaplessVecErr::OutOfBounds);
        }
        if index == self.used_data {
            return self.push_within_capacity(element);
        }

        for i in index..(self.used_data - 1) {
            self.internal_data.swap(i, i + 1);
        }

        self.used_data += 1;

        self.internal_data[index].write(element);

        Ok(())
    }

    pub fn find_index_of(&self, value: &Type) -> Option<usize>
    where
        Type: PartialEq,
    {
        let mut index = None;

        for (idx, v) in self.iter().enumerate() {
            if v == value {
                index = Some(idx);
                break;
            }
        }

        index
    }

    pub fn get(&self, index: usize) -> Result<&Type, HeaplessVecErr> {
        if index > self.used_data {
            return Err(HeaplessVecErr::OutOfBounds);
        }

        return Ok(unsafe { self.internal_data[index].assume_init_ref() });
    }

    pub fn get_mut(&mut self, index: usize) -> Result<&mut Type, HeaplessVecErr> {
        if index > self.used_data {
            return Err(HeaplessVecErr::OutOfBounds);
        }

        return Ok(unsafe { self.internal_data[index].assume_init_mut() });
    }

    pub const fn as_slice(&self) -> &[Type] {
        unsafe {
            core::slice::from_raw_parts(self.internal_data.as_ptr() as *const Type, self.used_data)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [Type] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.internal_data.as_mut_ptr() as *mut Type,
                self.used_data,
            )
        }
    }

    pub const fn as_ptr(&self) -> *const Type {
        self.internal_data.as_ptr() as *const Type
    }

    pub fn as_mut_ptr(&mut self) -> *mut Type {
        return self.internal_data.as_mut_ptr() as *mut Type;
    }

    pub fn iter(&self) -> Iter<Type> {
        self.as_slice().iter()
    }

    pub fn mut_iter(&mut self) -> IterMut<Type> {
        self.as_mut_slice().iter_mut()
    }
}

impl<Type, const SIZE: usize> Debug for HeaplessVec<Type, SIZE>
where
    Type: Sized,
    Type: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", &self.internal_data[0..self.used_data])
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
        assert_eq!(vec.get(0), Ok(&69));

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

        assert_eq!(*total.get(0).unwrap(), 10);
        for i in 1..11 {
            assert_eq!(*total.get(i as usize).unwrap(), i - 1);
        }
    }
}
