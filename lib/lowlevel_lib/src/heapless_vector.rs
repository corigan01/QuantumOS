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
use core::mem::{MaybeUninit};
use core::slice::{Iter, IterMut};

/// # Heapless Vector Error
/// This is the errors that heapless vector can encounter while using it.
///
/// Some Errors include:
///     * VectorFull
///     * OutOfBounds
///
/// ### What these errors mean?
///
/// `VectorFull` is probebly the most common error that you might encourter as it is called
/// when you try to push an element into a full vector. Since this vector is not heap allocated
/// and instead lives on the stack, it must have a known size know at compile time. This is
/// unfortunate as it limits the user to the size defined at compile time.
///
/// `OutOfBouds` is going to be another big one that users might encourter while using
/// this structure as it is returned when you try to access something in the vector that is
/// out of bounds of the valid data.
///
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum HeaplessVecErr {
    VectorFull,
    OutOfBounds,
}

/// # Heapless Vector
/// Heapless vector is my method on abstracting the use of raw type arrays that can be hard to
/// deal with and add elements to. So this is a heapless abstraction of that. Unfornatnly since
/// there is no heap allocator, we must live on the stack which has its limitations. These
/// limitations include having a size known at compile time which can be inconventent due to the
/// limitation of max size. However, its also bad for the other reason as if this vector is oversized
/// for the data, it still takes up all that data in 'preperation' for the addition of new data.
///
///
/// # Common Useage
/// ```rust
/// use quantum_lib::heapless_vector::HeaplessVec;
///
/// fn main() {
///     let mut  my_vector: HeaplessVec<usize, 10> = HeaplessVec::new();
///
///     my_vector.push_within_capacity(2).unwrap();
///     my_vector.push_within_capacity(6).unwrap();
///
///     assert_eq!(my_vector.get(0).unwrap(), &2_usize);
///     assert_eq!(my_vector.get(1).unwrap(), &6_usize);
///     assert_eq!(my_vector.len(), 2_usize);
///
/// }
pub struct HeaplessVec<Type, const SIZE: usize> {
    /// # Internal Data
    /// This is the raw array that stores the data with type `Type` and size `SIZE`.
    ///
    /// Rust is very convenient as it already has many helper functions in its most
    /// primitive types, which aids in the development in in this type.
    internal_data: [MaybeUninit<Type>; SIZE],

    /// # Used Data
    /// `used_data` represents the amount of data inside `internal_data`. This helps
    /// us know how many positions are populated and which ones are free for use.
    used_data: usize,
}

impl<Type, const SIZE: usize> HeaplessVec<Type, SIZE> {
    /// # New
    /// Construct a new empty heapless vector.
    ///
    pub fn new() -> Self {
        Self {
            internal_data: unsafe { mem::zeroed() },
            used_data: 0,
        }
    }

    /// # Len
    /// Gets the current length of the vector, or more precisely it will return the amount of
    /// elements that are currently used inside the vector. This is useful when you want to
    /// know how many elements have been populated inside the array.
    pub fn len(&self) -> usize {
        self.used_data
    }

    /// # Capacity
    /// Gets the current max length of this vector. The maxium amount of elements that can
    /// be allocated in this type at one time. If anymore elements are allocated with
    /// `push_within_capsity` then it will return `VectorFull`.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        SIZE
    }

    /// # Push within Capacity
    /// Gets a value from the user and will append it to the end of the vector. This will also
    /// increase the len() of the vector to represent the newly added data.
    pub fn push_within_capacity(&mut self, value: Type) -> Result<(), HeaplessVecErr> {
        if self.used_data >= self.internal_data.len() {
            return Err(HeaplessVecErr::VectorFull);
        }

        let index = self.used_data;
        self.internal_data[index].write(value);

        self.used_data += 1;

        Ok(())
    }

    /// # Pop
    /// Pops (removes) the last element in the array. If there are no elements in the array,
    /// then it will simply return `None`; however if there are elements in the array then it
    /// return `Some(Type)` with the data that it removed from the vector
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

    pub fn retain<Function>(&mut self, runner: &mut Function)
        where Function: FnMut(&Type) -> bool {

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

    /// # Get
    /// Gets the data at the index provided. If the index is invalid or out of bounds, then it will
    /// return `Err(OutOfBounds)`. Otherwise it will return a reference to the data with the same
    /// lifetime as self.
    pub fn get(&self, index: usize) -> Result<&Type, HeaplessVecErr> {
        if index > self.used_data {
            return Err(HeaplessVecErr::OutOfBounds);
        }

        return Ok(unsafe { self.internal_data[index].assume_init_ref() });
    }

    /// # Get Mut
    /// Gets the data at the index provided, but mutable. If the index is invalid or out of bounds,
    /// then it will return `Err(OutOfBounds)` just like get. Otherwise it will return a mutable
    /// reference to the data with the same lifetime as self.
    pub fn get_mut(&mut self, index: usize) -> Result<&mut Type, HeaplessVecErr> {
        if index > self.used_data {
            return Err(HeaplessVecErr::OutOfBounds);
        }

        return Ok(unsafe { self.internal_data[index].assume_init_mut() });
    }

    /// # As Slice
    /// Returns the raw internal data for minupulation.
    pub fn as_slice(&self) -> &[Type] {
        unsafe {
            core::slice::from_raw_parts(
                self.internal_data.as_ptr() as *const Type,
                self.used_data
            )
        }
    }

    /// # As Mut Slice
    /// Returns the raw internal data for minupulation but mutable with the data inside this
    /// vector.
    ///
    /// # Safety
    /// Because its returning a mutable reference to the data thats included in the vector,
    /// we have no way of updating the size if you go out of bounds from the orignal size.
    ///
    /// So its up to the caller to not add elements to the array if possible. If however,
    /// elements are added to interal_data, we will not repersent the new data and override it
    /// as soon as the caller calls `push`. Furthermore, a call to get will return `OutOfBounds`
    /// if data was manipulated out of bounds from len.
    pub fn as_mut_slice(&mut self) -> &mut [Type] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.internal_data.as_mut_ptr() as *mut Type,
                self.used_data
            )
        }
    }

    /// # As Ptr
    /// return a ptr to the internal data.
    ///
    /// # Safey
    /// It is up to the caller to ensure that safety is used when hanlding raw ptrs. Since simply
    /// returning a ptr is not considered 'unsafe' it is not required that this method is marked
    /// unsafe as it doesn't directly cause any unsafe behavour. However, the caller can derefrence
    /// this ptr which will cause issues if the data is malformed.
    pub fn as_ptr(&self) -> *const Type {
        return self.internal_data.as_ptr() as *const Type;
    }

    /// # As Mut Ptr
    /// return a mut ptr to the internal data.
    ///
    /// # Safey
    /// It is up to the caller to ensure that safety is used when hanlding raw ptrs. Since simply
    /// returning a ptr is not considered 'unsafe' it is not required that this method is marked
    /// unsafe as it doesn't directly cause any unsafe behavour. However, the caller can derefrence
    /// this ptr which will cause issues if the data is malformed.
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
}