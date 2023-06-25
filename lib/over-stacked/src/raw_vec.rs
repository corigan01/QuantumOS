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
use core::mem::MaybeUninit;
use core::slice::{Iter, IterMut};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RawVecErr {
    NotEnoughMem,
    OutOfBounds,
    Invalid
}

pub struct RawVec<'a, Type> {
    data: &'a mut [MaybeUninit<Type>],
    len: usize
}


impl<'a, Type> RawVec<'a, Type> {
    pub fn uninit() -> Self {
        Self {
            data: &mut [MaybeUninit::uninit(); 0],
            len: 0
        }
    }

    pub fn new(data: &'a mut [MaybeUninit<Type>]) -> Self {
        Self {
            data,
            len: 0
        }
    }

    pub fn new_data_backend(&mut self, data: &'a mut [MaybeUninit<Type>]) -> Result<(), RawVecErr> {
        if data.len() < self.len {
            return Err(RawVecErr::Invalid);
        }

        for i in 0..self.len {
            let copied_value = unsafe { self.data[i].assume_init_read() };
            data[i].write(copied_value);
        }

        self.data = data;

        Ok(())
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn capacity(&self) -> usize {
        self.data.len()
    }

    pub fn push_within_capacity(&mut self, value: Type) -> Result<(), RawVecErr> {
        if self.len >= self.data.len() {
            return Err(RawVecErr::NotEnoughMem);
        }

        let index = self.len;
        self.data[index].write(value);

        self.len += 1;

        Ok(())
    }

    pub fn pop(&mut self) -> Option<Type> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        Some(unsafe { self.data[self.len + 1].assume_init_read() })
    }

    pub fn remove(&mut self, index: usize) -> Option<Type> {
        if self.len == 0 && index > self.len {
            return None;
        }

        let moved_out_data = unsafe { self.data[index].assume_init_read() };

        for i in index..(self.len - 1) {
            self.data.swap(i, i + 1);
        }

        self.len -= 1;

        Some(moved_out_data)
    }

    pub fn push_vec(&mut self, rhs: &RawVec<Type>) -> Result<(), RawVecErr> {
        for rhs_index in 0..rhs.len {
            unsafe {
                self.push_within_capacity(rhs.data[rhs_index].assume_init_read())?;
            }
        }

        Ok(())
    }

    pub fn retain<Function>(&mut self, mut runner: Function)
        where
            Function: FnMut(&Type) -> bool,
    {
        let data_ptr = self.data.as_ptr();
        let mut current_iterating_index = 0;

        loop {
            if current_iterating_index > self.len {
                break;
            }

            let data_ref = unsafe { (&*data_ptr).assume_init_ref() };
            let should_keep = runner(data_ref);

            if !should_keep {
                self.remove(current_iterating_index).unwrap();
                continue;
            }

            current_iterating_index += 1;
        }
    }

    pub fn insert(&mut self, index: usize, element: Type) -> Result<(), RawVecErr> {
        if self.len >= self.data.len() {
            return Err(RawVecErr::NotEnoughMem);
        }
        if index > self.len {
            return Err(RawVecErr::NotEnoughMem);
        }
        if index == self.len {
            return self.push_within_capacity(element);
        }

        for i in index..(self.len - 1) {
            self.data.swap(i, i + 1);
        }

        self.len += 1;

        self.data[index].write(element);

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

    pub fn get(&self, index: usize) -> Option<&Type> {
        if index > self.len {
            return None;
        }

        return Some(unsafe { self.data[index].assume_init_ref() });
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Type> {
        if index > self.len {
            return None;
        }

        return Some(unsafe { self.data[index].assume_init_mut() });
    }

    pub const fn as_slice(&self) -> &[Type] {
        unsafe {
            core::slice::from_raw_parts(self.data.as_ptr() as *const Type, self.len)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [Type] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.data.as_mut_ptr() as *mut Type,
                self.len,
            )
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut Type {
        return self.data.as_mut_ptr() as *mut Type;
    }

    pub fn iter(&self) -> Iter<Type> {
        self.as_slice().iter()
    }

    pub fn mut_iter(&mut self) -> IterMut<Type> {
        self.as_mut_slice().iter_mut()
    }
}

impl<'a, Type> Debug for RawVec<'a, Type>
    where
        Type: Sized,
        Type: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", &self.data[0..self.len])
    }
}

impl<'a, Type> Default for RawVec<'a, Type> {
    fn default() -> Self {
        Self::uninit()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_small_raw_vec() {
        let mut backend_data: [MaybeUninit<i32>; 10] = [MaybeUninit::uninit(); 10];
        let mut raw_vec = RawVec::new(&mut backend_data);

        assert_eq!(raw_vec.push_within_capacity(10), Ok(()));
        assert_eq!(raw_vec.push_within_capacity(9), Ok(()));

        assert_eq!(raw_vec.get(0), Some(&10));
        assert_eq!(raw_vec.get(1), Some(&9));
    }

    #[test]
    fn test_lots_of_raw_vec() {
        let mut backend_data: [MaybeUninit<i32>; 10] = [MaybeUninit::uninit(); 10];
        let mut raw_vec = RawVec::new(&mut backend_data);

        for i in 0..10 {
            assert_eq!(raw_vec.push_within_capacity(i), Ok(()));
        }

        for i in 0..10 {
            assert_eq!(raw_vec.get(i), Some(&(i as i32)));
        }

    }

    #[test]
    fn test_removing_items() {
        let mut backend_data: [MaybeUninit<i32>; 10] = [MaybeUninit::uninit(); 10];
        let mut raw_vec = RawVec::new(&mut backend_data);

        for i in 0..10 {
            assert_eq!(raw_vec.push_within_capacity(i), Ok(()));
        }

        assert_ne!(raw_vec.push_within_capacity(8008), Ok(()));

        assert_eq!(raw_vec.remove(0), Some(0));
    }


}