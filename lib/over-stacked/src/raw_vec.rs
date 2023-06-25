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
use core::ops::{Index, IndexMut};
use core::ptr::NonNull;
use core::slice::{Iter, IterMut};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RawVecErr {
    NotEnoughMem,
    OutOfBounds,
    Invalid
}

pub struct RawVec<Type> {
    data: NonNull<Type>,
    total: usize,
    used: usize
}


impl<Type> RawVec<Type> {
    pub fn new() -> Self {
        Self {
            data: NonNull::dangling(),
            total: 0,
            used: 0,
        }
    }

    pub fn begin(ptr: NonNull<Type>, size: usize) -> Self {
        Self {
            data: ptr,
            total: size,
            used: 0
        }
    }

    pub unsafe fn without_checking_set_new_ptr(&mut self, ptr: *mut Type) {
        self.data = NonNull::new_unchecked(ptr);
    }

    pub fn grow(&mut self, new_ptr: NonNull<Type>, total_size: usize) -> Result<NonNull<Type>, RawVecErr> {
        if total_size < self.used {
            return Err(RawVecErr::Invalid);
        }

        for i in 0..self.used {
            let data_copy = self.read(i).unwrap();
            *(unsafe {&mut *new_ptr.as_ptr().add(i) }) = data_copy;
        }

        let return_ptr = self.data;

        self.data = new_ptr;
        self.total = total_size;

        Ok(return_ptr)
    }

    pub fn get_ref(&self, index: usize) -> Option<&Type> {
        if index >= self.used {
            return None;
        }

        Some(unsafe { &*self.data.as_ptr().add(index) })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Type> {
        if index >= self.used {
            return None;
        }

        Some(unsafe {&mut *self.data.as_ptr().add(index)})
    }

    pub fn capacity(&self) -> usize {
        self.total
    }

    pub fn len(&self) -> usize {
        self.used
    }

    pub(crate) fn read(&self, index: usize) -> Option<Type> {
        if index >= self.used || index >= self.total {
            return None;
        }

        Some(unsafe { self.data.as_ptr().add(index).read_unaligned() })
    }

    pub(crate) fn store(&mut self, index: usize, data: Type) -> Result<(), RawVecErr> {
        if index >= self.total {
            return Err(RawVecErr::OutOfBounds);
        }

        *(unsafe { &mut *self.data.as_ptr().add(index) }) = data;

        Ok(())
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        assert!(a < self.used);
        assert!(b < self.used);

        let a_copy = self.read(a).unwrap();
        self.store(a, self.read(b).unwrap()).unwrap();
        self.store(b, a_copy).unwrap();
    }

    pub fn push_within_capacity(&mut self, value: Type) -> Result<(), RawVecErr> {
        if self.used >= self.total {
            return Err(RawVecErr::NotEnoughMem);
        }

        self.store(self.used, value)?;
        self.used += 1;

        Ok(())
    }

    pub fn pop(&mut self) -> Option<Type> {
        let data = self.read(self.used - 1)?;
        self.used -= 1;

        Some(data)
    }

    pub fn remove(&mut self, index: usize) -> Option<Type> {
        if index >= self.used {
            return None;
        }

        let moved_out_data = self.read(index)?;

        for i in index..(self.used - 1) {
            self.swap(i, i + 1)
        }

        self.used -= 1;

        Some(moved_out_data)
    }

    pub fn push_vec(&mut self, rhs: RawVec<Type>) -> Result<(), RawVecErr> {
        if rhs.used + self.used > self.total {
            return Err(RawVecErr::NotEnoughMem);
        }

        for i in 0..rhs.used {
            self.push_within_capacity(rhs.read(i).unwrap())?;
        }

        Ok(())
    }

    pub fn retain<Function>(&mut self, mut runner: Function) where Function: FnMut(&Type) -> bool {
        let mut how_many_removed = 0;

        for i in 0..self.used {
            let data_ref = self.get_ref(i - how_many_removed).unwrap();
            let should_keep = runner(data_ref);

            if !should_keep {
                self.remove(i - how_many_removed);

                how_many_removed += 1;
            }
        }
    }

    pub fn insert(&mut self, index: usize, a: Type) -> Result<(), RawVecErr> {
        if self.used >= self.total {
            return Err(RawVecErr::NotEnoughMem);
        }
        if index > self.used {
            return Err(RawVecErr::OutOfBounds);
        }

        for i in (index..self.used).rev() {
            self.store(i + 1, self.read(i).unwrap()).unwrap();
        }

        self.used += 1;
        self.store(index, a)?;

        Ok(())
    }

    pub fn find_index_of(&self, a: &Type) -> Option<usize>
        where Type: PartialEq {

        for i in 0..self.used {
            let data_ref = self.get_ref(i)?;

            if data_ref == a {
                return Some(i);
            }
        }

        None
    }

    pub fn as_slice(&self) -> &[Type] {
        unsafe {
            core::slice::from_raw_parts(self.data.as_ptr(), self.used)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [Type] {
        unsafe {
            core::slice::from_raw_parts_mut(self.data.as_ptr(), self.used)
        }
    }

    pub fn as_ptr(&self) -> *const Type {
        self.data.as_ptr()
    }

    pub unsafe fn as_mut(&mut self) -> *mut Type {
        self.data.as_mut()
    }

    pub fn iter(&self) -> Iter<Type> {
        self.as_slice().iter()
    }

    pub fn mut_iter(&mut self) -> IterMut<Type> {
        self.as_mut_slice().iter_mut()
    }
}

impl<Type> Index<usize> for RawVec<Type> {
    type Output = Type;

    fn index(&self, index: usize) -> &Self::Output {
        self.get_ref(index).unwrap()
    }
}

impl<Type> IndexMut<usize> for RawVec<Type> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<Type> Debug for RawVec<Type> where Type: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.as_slice())
        } else {
            write!(f, "{:?}", self.as_slice())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_ptr_stay_the_same_after_init() {
        let mut data_storage = [0_i32; 10];
        let mut raw_vec = RawVec::new();

        assert!(raw_vec.grow(
            NonNull::new(data_storage.as_mut_ptr()).unwrap(),
            data_storage.len()
        ).is_ok(), "RawVec grow failed!");

        assert_eq!(raw_vec.as_ptr(), data_storage.as_ptr());
    }

    #[test]
    fn test_adding_elements_to_raw_vector() {
        let mut data_storage = [0_i32; 10];
        let mut raw_vec = RawVec::new();

        assert!(raw_vec.grow(
            NonNull::new(data_storage.as_mut_ptr()).unwrap(),
            data_storage.len()
        ).is_ok(), "RawVec grow failed!");

        assert!(raw_vec.push_within_capacity(69).is_ok(),
            "Push '69' into RawVec failed!");

        assert_eq!(raw_vec.get_ref(0), Some(&69),
            "Get '69' from RawVec failed!");
    }

    #[test]
    fn test_adding_tons_of_elements() {
        let mut data_storage = [0_i32; 10];
        let mut raw_vec = RawVec::new();

        assert!(raw_vec.grow(
            NonNull::new(data_storage.as_mut_ptr()).unwrap(),
            data_storage.len()
        ).is_ok(), "RawVec grow failed!");

        for i in 0..10 {
            assert!(raw_vec.push_within_capacity(i).is_ok(),
                    "Push '{}' into RawVec failed!", i);
        }

        assert!(raw_vec.push_within_capacity(11).is_err(),
                "RawVec allowed us to push 11 elements into a 10 element array!");

        for i in 0..10 {
            assert_eq!(raw_vec.get_ref(i), Some(&(i as i32)),
                       "Get '{}' from RawVec failed!", i);
        }

        assert!(raw_vec.get_ref(11).is_none(),
                "RawVec allowed us to access index '11' from a 10 element array");
    }

    #[test]
    fn test_swap_elements_in_raw_vector() {
        let mut data_storage = [0_i32; 10];
        let mut raw_vec = RawVec::new();

        assert!(raw_vec.grow(
            NonNull::new(data_storage.as_mut_ptr()).unwrap(),
            data_storage.len()
        ).is_ok(), "RawVec grow failed!");

        assert!(raw_vec.push_within_capacity(1).is_ok());
        assert!(raw_vec.push_within_capacity(2).is_ok());

        raw_vec.swap(0, 1);

        assert_eq!(raw_vec.get_ref(0), Some(&2));
        assert_eq!(raw_vec.get_ref(1), Some(&1));
    }

    #[test]
    fn test_pop_raw_vec() {
        let mut data_storage = [0_i32; 10];
        let mut raw_vec = RawVec::new();

        assert!(raw_vec.grow(
            NonNull::new(data_storage.as_mut_ptr()).unwrap(),
            data_storage.len()
        ).is_ok(), "RawVec grow failed!");

        assert!(raw_vec.push_within_capacity(1).is_ok());
        assert!(raw_vec.push_within_capacity(2).is_ok());

        assert_eq!(raw_vec.pop(), Some(2));
        assert_eq!(raw_vec.len(), 1);
    }

    #[test]
    fn test_insert_element_raw_vec() {
        let mut data_storage = [0_i32; 10];
        let mut raw_vec = RawVec::new();

        assert!(raw_vec.grow(
            NonNull::new(data_storage.as_mut_ptr()).unwrap(),
            data_storage.len()
        ).is_ok(), "RawVec grow failed!");

        assert!(raw_vec.push_within_capacity(1).is_ok());
        assert!(raw_vec.push_within_capacity(3).is_ok());
        assert_eq!(raw_vec.len(), 2);

        assert!(raw_vec.insert(1, 2).is_ok());

        assert_eq!(raw_vec.get_ref(0), Some(&1));
        assert_eq!(raw_vec.get_ref(1), Some(&2));
        assert_eq!(raw_vec.get_ref(2), Some(&3));
        assert_eq!(raw_vec.get_ref(3), None);
    }

    #[test]
    fn test_adding_pushing_new_vector() {
        let mut data_storage1 = [0_i32; 10];
        let mut raw_vec1 = RawVec::new();

        assert!(raw_vec1.grow(
            NonNull::new(data_storage1.as_mut_ptr()).unwrap(),
            data_storage1.len()
        ).is_ok(), "RawVec1 grow failed!");

        let mut data_storage2 = [0_i32; 10];
        let mut raw_vec2 = RawVec::new();

        assert!(raw_vec2.grow(
            NonNull::new(data_storage2.as_mut_ptr()).unwrap(),
            data_storage2.len()
        ).is_ok(), "RawVec2 grow failed!");

        assert!(raw_vec1.push_within_capacity(1).is_ok());
        assert!(raw_vec1.push_within_capacity(2).is_ok());
        assert_eq!(raw_vec1.len(), 2);

        assert!(raw_vec2.push_within_capacity(3).is_ok());
        assert!(raw_vec2.push_within_capacity(4).is_ok());
        assert_eq!(raw_vec2.len(), 2);

        raw_vec1.push_vec(raw_vec2).unwrap();

        assert_eq!(raw_vec1.len(), 4);

        assert_eq!(raw_vec1.get_ref(0), Some(&1));
        assert_eq!(raw_vec1.get_ref(1), Some(&2));
        assert_eq!(raw_vec1.get_ref(2), Some(&3));
        assert_eq!(raw_vec1.get_ref(3), Some(&4));
        assert_eq!(raw_vec1.get_ref(4), None);
    }

    #[test]
    fn test_removing_elements_from_raw_vec() {
        let mut data_storage = [0_i32; 10];
        let mut raw_vec = RawVec::new();

        assert!(raw_vec.grow(
            NonNull::new(data_storage.as_mut_ptr()).unwrap(),
            data_storage.len()
        ).is_ok(), "RawVec grow failed!");

        assert!(raw_vec.push_within_capacity(1).is_ok());
        assert!(raw_vec.push_within_capacity(2).is_ok());
        assert!(raw_vec.push_within_capacity(3).is_ok());
        assert!(raw_vec.push_within_capacity(9).is_ok());
        assert!(raw_vec.push_within_capacity(4).is_ok());
        assert_eq!(raw_vec.len(), 5);

        assert_eq!(raw_vec.remove(3), Some(9));
        assert_eq!(raw_vec.len(), 4);

        assert_eq!(raw_vec.as_slice(), &[1, 2, 3, 4]);
    }
}


