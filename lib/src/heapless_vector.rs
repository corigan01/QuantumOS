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
use core::ops::Index;

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
/// unfornatate as it limites the user to the size defined at compile time.
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
///     let my_vector: HeaplessVec<usize, 10> = HeaplessVec::new();
///
///     my_vector.push_within_capsity(2).unwrap();
///     my_vector.push_within_capsity(6).unwrap();
///
///     assert_eq!(my_vector.get(0).unwrap(), 2);
///     assert_eq!(my_vector.get(1).unwrap(), 6);
///     assert_eq!(my_vector.len(), 2);
///
/// }
pub struct HeaplessVec<Type, const SIZE: usize> {
    /// # Internal Data
    /// This is the raw array that stores the data with type `Type` and size `SIZE`.
    ///
    /// Rust is very conventent as it already has many helper functions in its most
    /// primitive types, which aids in the development in in this type.
    internal_data: [Type; SIZE],

    /// # Used Data
    /// `used_data` repersents the amount of data inside `internal_data`. This helps
    /// us know how many positions are populated and which ones are free for use.
    used_data: usize,
}

impl<Type, const SIZE: usize> HeaplessVec<Type, SIZE>
where
    Type: Default,
    Type: Copy,
{
    /// # New
    /// Construct a new empty heapless vector.
    ///
    /// # Required Types
    /// Your type must include `Default` and `Copy` because this vector needs to be pre-filled
    /// with some data even though the user 'cant' access it. This is because of rusts saftey and
    /// type system.
    ///
    pub fn new() -> Self {
        Self {
            internal_data: [Default::default(); SIZE],
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

    /// # Push within Capsity
    /// Gets a value from the user and will append it to the end of the vector. This will also
    /// increase the len() of the vector to repersent the newly added data.
    pub fn push_within_capsity(&mut self, value: Type) -> Result<(), HeaplessVecErr> {
        if self.used_data >= self.internal_data.len() {
            return Err(HeaplessVecErr::VectorFull);
        }

        let index = self.used_data;
        self.internal_data[index] = value;

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

        Some(self.internal_data[self.used_data + 1])
    }

    /// # Get
    /// Gets the data at the index provided. If the index is invalid or out of bounds, then it will
    /// return `Err(OutOfBounds)`. Otherwise it will return a reference to the data with the same
    /// lifetime as self.
    pub fn get<'a>(&'a self, index: usize) -> Result<&'a Type, HeaplessVecErr> {
        if index > self.used_data {
            return Err(HeaplessVecErr::OutOfBounds);
        }

        return Ok(&self.internal_data[index]);
    }

    /// # Get Mut
    /// Gets the data at the index provided, but mutable. If the index is invalid or out of bounds,
    /// then it will return `Err(OutOfBounds)` just like get. Otherwise it will return a mutable
    /// reference to the data with the same lifetime as self.
    pub fn get_mut<'a>(&'a mut self, index: usize) -> Result<&'a mut Type, HeaplessVecErr> {
        if index > self.used_data {
            return Err(HeaplessVecErr::OutOfBounds);
        }

        return Ok(&mut self.internal_data[index]);
    }

    /// # As Slice
    /// Returns the raw internal data for minupulation.
    pub fn as_slice<'a>(&'a self) -> &'a [Type] {
        self.internal_data.as_slice()
    }

    /// # As Mut Slice
    /// Returns the raw internal data for minupulation but mutable with the data inside this
    /// vector.
    ///
    /// # Saftey
    /// Because its returning a mutable reference to the data thats included in the vector,
    /// we have no way of updating the size if you go out of bounds from the orignal size.
    ///
    /// So its up to the caller to not add elements to the array if possible. If however,
    /// elements are added to interal_data, we will not repersent the new data and override it
    /// as soon as the caller calls `push`. Furthermore, a call to get will return `OutOfBounds`
    /// if data was miniupulated out of bounds from len.
    pub unsafe fn as_mut_slice<'a>(&'a mut self) -> &'a mut [Type] {
        self.internal_data.as_mut_slice()
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
        return self.internal_data.as_ptr();
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
        return self.internal_data.as_mut_ptr();
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

pub struct HeaplessVecIter<Type, const SIZE: usize> {
    vec: HeaplessVec<Type, SIZE>,
    index: usize,
}

impl<Type, const SIZE: usize> IntoIterator for HeaplessVec<Type, SIZE>
where
    Type: Default,
    Type: Copy,
{
    type IntoIter = HeaplessVecIter<Type, SIZE>;
    type Item = Type;

    fn into_iter(self) -> Self::IntoIter {
        return HeaplessVecIter {
            vec: self,
            index: 0,
        };
    }
}

impl<Type, const SIZE: usize> Iterator for HeaplessVecIter<Type, SIZE>
where
    Type: Default,
    Type: Copy,
{
    type Item = Type;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.vec.used_data {
            return None;
        }

        let data = self.vec.internal_data[self.index];
        self.index += 1;

        return Some(data);
    }
}

impl<Type, const SIZE: usize> Index<usize> for HeaplessVec<Type, SIZE>
where
    Type: Default,
    Type: Copy,
    Type: Debug,
    Type: Sized,
{
    type Output = Type;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Index out of bounds!")
    }
}
