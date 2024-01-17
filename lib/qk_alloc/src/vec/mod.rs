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

use crate::boxed::Box;
use crate::heap::{AllocatorAPI, GlobalAlloc};
use crate::vec::raw_vec::RawVec;
use core::cmp::Ordering;
use core::fmt::{Debug, Formatter};
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::ptr;

mod raw_vec;

pub struct Vec<Type, Alloc: AllocatorAPI = GlobalAlloc> {
    raw: RawVec<Type, Alloc>,
    len: usize,
}

impl<Type> Vec<Type, GlobalAlloc> {
    pub const fn new() -> Self {
        Self::new_in(GlobalAlloc)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_in(capacity, GlobalAlloc)
    }

    pub unsafe fn from_raw_parts(ptr: *mut Type, len: usize, capacity: usize) -> Self {
        Self {
            raw: RawVec::from_raw_parts(ptr, capacity),
            len,
        }
    }

    pub fn extend_from_slice(&mut self, slice: &[Type])
    where
        Type: Clone,
    {
        for val in slice {
            self.push(val.clone());
        }
    }
}

impl<Type, Alloc: AllocatorAPI> Vec<Type, Alloc> {
    pub const fn new_in(alloc: Alloc) -> Self {
        Self {
            raw: RawVec::new_in(alloc),
            len: 0,
        }
    }

    pub fn with_capacity_in(capacity: usize, alloc: Alloc) -> Self {
        Self {
            raw: RawVec::with_capacity_in(capacity, alloc),
            len: 0,
        }
    }

    pub const fn as_ptr(&self) -> *const Type {
        self.raw.ptr.as_ptr() as *const Type
    }

    pub fn as_mut_ptr(&mut self) -> *mut Type {
        self.raw.ptr.as_ptr()
    }

    pub fn as_slice<'a>(&self) -> &'a [Type] {
        unsafe { core::slice::from_raw_parts(self.raw.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_slice<'a>(&self) -> &'a mut [Type] {
        unsafe { core::slice::from_raw_parts_mut(self.raw.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_uninit_slice<'a>(&mut self) -> &'a mut [MaybeUninit<Type>] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.raw.ptr.as_ptr() as *mut MaybeUninit<Type>,
                self.raw.cap,
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

    pub fn append(&mut self, other: &mut Vec<Type, Alloc>) {
        while let Some(value) = other.pop_front() {
            self.push(value);
        }
    }

    pub fn pop_front(&mut self) -> Option<Type> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        let read;
        unsafe {
            read = ptr::read(self.as_mut_ptr());
            ptr::copy(self.as_mut_ptr().add(1), self.as_mut_ptr(), self.len);
        }

        Some(read)
    }

    pub fn pop(&mut self) -> Option<Type> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        Some(unsafe { ptr::read(self.as_ptr().add(self.len)) })
    }

    pub fn insert(&mut self, index: usize, value: Type) {
        assert!(
            index <= self.len,
            "Index out of bounds! Got index {}, but len is only {}",
            index,
            self.len
        );

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

    pub fn replace(&mut self, index: usize, value: Type) -> Option<Type> {
        assert!(
            index <= self.len,
            "Index out of bounds! Got index {}, but len is only {}",
            index,
            self.len
        );

        if self.should_reserve() {
            self.reserve(1);
        }

        let return_value;
        unsafe {
            let ptr = self.as_mut_ptr().add(index);

            if index >= self.len {
                ptr::write(ptr, value);
                return None;
            }

            return_value = ptr::read(ptr);
            ptr::write(ptr, value);
        }

        Some(return_value)
    }

    pub fn retain<F>(&mut self, mut should_keep: F)
    where
        F: FnMut(&Type) -> bool,
    {
        let mut index = 0;
        loop {
            if index >= self.len {
                return;
            }

            let current = unsafe { &*self.as_ptr().add(index) };

            if !should_keep(current) {
                self.remove(index);
                continue;
            }

            index += 1;
        }
    }

    pub fn remove_all(&mut self) {
        while let Some(_) = self.pop() {}
    }

    pub fn remove(&mut self, index: usize) -> Type {
        assert!(
            index <= self.len || self.len == 0,
            "Index out of bounds! Got index {}, but len is only {}",
            index,
            self.len
        );

        let return_value;
        unsafe {
            let ptr = self.as_mut_ptr().add(index);

            return_value = ptr::read(ptr);

            if self.len != 0 && self.len != index {
                ptr::copy(ptr.add(1), ptr, self.len - index - 1);
            }
        }

        self.len -= 1;
        return_value
    }

    pub fn get(&self, index: usize) -> Option<&Type> {
        if index > self.len() {
            return None;
        }

        let value_ref = unsafe { &*self.raw.ptr.as_ptr().add(index) };

        Some(value_ref)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Type> {
        if index > self.len() {
            return None;
        }

        let value_mut = unsafe { &mut *self.raw.ptr.as_ptr().add(index) };

        Some(value_mut)
    }

    pub fn sort(&mut self)
    where
        Type: Ord,
    {
        self.as_mut_slice().sort_unstable()
    }

    pub fn sort_by<F>(&mut self, function: F)
    where
        F: FnMut(&Type, &Type) -> Ordering,
    {
        self.as_mut_slice().sort_unstable_by(function)
    }
}

pub struct IntoIter<Type, Alloc: AllocatorAPI> {
    vec: Vec<Type, Alloc>,
}

impl<Type, Alloc: AllocatorAPI> IntoIter<Type, Alloc> {
    pub fn new(vec: Vec<Type, Alloc>) -> Self {
        Self { vec }
    }
}

impl<Type, Alloc: AllocatorAPI> Iterator for IntoIter<Type, Alloc> {
    type Item = Type;

    fn next(&mut self) -> Option<Self::Item> {
        self.vec.pop_front()
    }
}

impl<Type, Alloc: AllocatorAPI> IntoIterator for Vec<Type, Alloc> {
    type Item = Type;
    type IntoIter = IntoIter<Type, Alloc>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<Type> From<Box<[Type]>> for Vec<Type> {
    fn from(value: Box<[Type]>) -> Self {
        let mut b = ManuallyDrop::new(value);

        unsafe { Self::from_raw_parts(b.as_mut_ptr(), b.len(), b.len()) }
    }
}

impl<Type> From<&[Type]> for Vec<Type>
where
    Type: Clone,
{
    fn from(value: &[Type]) -> Self {
        let mut new_vec = Vec::with_capacity(value.len());
        for item in value {
            new_vec.push(item.clone());
        }

        new_vec
    }
}

impl<Type, const SIZE: usize> From<[Type; SIZE]> for Vec<Type> {
    fn from(value: [Type; SIZE]) -> Self {
        let mut vec = Vec::with_capacity(value.len());
        for item in value {
            vec.push(item);
        }

        vec
    }
}

impl<Type, Alloc: AllocatorAPI> Deref for Vec<Type, Alloc> {
    type Target = [Type];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<Type, Alloc: AllocatorAPI> DerefMut for Vec<Type, Alloc> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<Type> Clone for Vec<Type>
where
    Type: Clone,
{
    fn clone(&self) -> Self {
        let mut new = Vec::new();
        for i in self.iter() {
            new.push(i.clone());
        }

        new
    }
}

impl<Type, Alloc: AllocatorAPI> Debug for Vec<Type, Alloc>
where
    Type: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.as_slice())?;
        } else {
            write!(f, "{:?}", self.as_slice())?;
        }

        Ok(())
    }
}

impl<Type, Alloc: AllocatorAPI> Index<usize> for Vec<Type, Alloc> {
    type Output = Type;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Index out of bounds!")
    }
}

impl<Type, Alloc: AllocatorAPI> IndexMut<usize> for Vec<Type, Alloc> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("Index out of bounds!")
    }
}

impl<Type> FromIterator<Type> for Vec<Type> {
    fn from_iter<T: IntoIterator<Item = Type>>(iter: T) -> Self {
        let mut tmp = Vec::new();
        for i in iter {
            tmp.push(i);
        }

        tmp
    }
}

impl<'a, Type> FromIterator<&'a Type> for Vec<Type>
where
    Type: Clone + 'a,
{
    fn from_iter<T: IntoIterator<Item = &'a Type>>(iter: T) -> Self {
        let mut tmp = Vec::new();
        for i in iter {
            tmp.push(i.clone());
        }

        tmp
    }
}

impl<Type, Alloc: AllocatorAPI> Drop for Vec<Type, Alloc> {
    fn drop(&mut self) {
        self.remove_all();
    }
}

impl<Type> Default for Vec<Type> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn from_elem<Type: Clone>(element: Type, size: usize) -> Vec<Type> {
    let mut vector = Vec::with_capacity(size);

    for _ in 0..size {
        vector.push(element.clone());
    }

    vector
}

pub trait ToVec<Type> {
    fn to_vec(self) -> Vec<Type>;
}

impl<Type> ToVec<Type> for &[Type]
where
    Type: Clone,
{
    fn to_vec(self) -> Vec<Type> {
        self.iter().collect()
    }
}

impl<Type, const SIZE: usize> ToVec<Type> for [Type; SIZE] {
    fn to_vec(self) -> Vec<Type> {
        self.into_iter().collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::heap::set_example_allocator;
    use crate::string::String;

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

    #[test]
    fn test_sorting_elements() {
        set_example_allocator(4096);
        let mut vector: Vec<i32> = Vec::new();

        vector.push(3);
        vector.push(1);
        vector.push(0);
        vector.push(2);

        vector.sort();

        assert_eq!(vector[0], 0);
        assert_eq!(vector[1], 1);
        assert_eq!(vector[2], 2);
        assert_eq!(vector[3], 3);
    }

    #[test]
    fn test_replacing_vector_element() {
        set_example_allocator(4096);
        let mut vec: Vec<i32> = Vec::new();

        vec.push(0);
        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert_eq!(vec[0], 0);
        assert_eq!(vec[1], 1);
        assert_eq!(vec[2], 2);
        assert_eq!(vec[3], 3);

        vec.replace(0, 3);
        vec.replace(1, 2);
        vec.replace(2, 1);
        vec.replace(3, 0);

        assert_eq!(vec[0], 3);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 1);
        assert_eq!(vec[3], 0);
    }

    #[test]
    fn test_unaligned_u16_push_on_dirty_buffer() {
        set_example_allocator(4096);
        let _ = String::from(" ");
        let mut vec: Vec<u16> = Vec::new();

        for i in 0..256 {
            vec.push(i);
        }

        assert_eq!(vec.len(), 256);
    }

    #[test]
    fn test_retain() {
        set_example_allocator(4096);
        let mut vec: Vec<i32> = Vec::new();

        for i in 0..10 {
            vec.push(i);
        }

        vec.retain(|value| value % 2 == 0);

        assert_eq!(vec.as_slice(), &[0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_pop_front() {
        set_example_allocator(4096);
        let mut vec: Vec<i32> = Vec::new();

        for i in 0..10 {
            vec.push(i);
        }
    }

    #[test]
    fn test_from() {
        set_example_allocator(4096);
        let vec = Vec::from([0, 1, 2, 3, 4]);

        for i in 0..5 {
            assert_eq!(vec[i], i);
        }
    }
}
