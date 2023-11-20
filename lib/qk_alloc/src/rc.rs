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

use crate::{
    boxed::Box,
    heap::{AllocatorAPI, GlobalAlloc},
};
use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

struct RcBox<Type: ?Sized> {
    strong_count: NonNull<usize>,
    weak_count: NonNull<usize>,
    value: Type,
}

pub struct Rc<Type: ?Sized, Alloc: AllocatorAPI = GlobalAlloc> {
    val: NonNull<RcBox<Type>>,
    ph: PhantomData<Alloc>,
    alloc: Alloc,
}

// Rust Doc -- Remove access to send
// impl<Type: ?Sized, Alloc: AllocatorAPI> !Send for Rc<Type, Alloc> {}

impl<Type: ?Sized, Alloc: AllocatorAPI> Rc<Type, Alloc> {
    #[inline]
    fn inner(&self) -> &RcBox<Type> {
        unsafe { self.val.as_ref() }
    }

    #[inline]
    unsafe fn from_inner_in(inner: NonNull<RcBox<Type>>, alloc: Alloc) -> Self {
        Self {
            val: inner,
            ph: PhantomData,
            alloc,
        }
    }

    #[inline]
    unsafe fn from_ptr_in(ptr: *mut RcBox<Type>, alloc: Alloc) -> Self {
        Self {
            val: NonNull::new_unchecked(ptr),
            ph: PhantomData,
            alloc,
        }
    }
}

impl<Type: ?Sized> Rc<Type> {
    #[inline]
    unsafe fn from_inner(inner: NonNull<RcBox<Type>>) -> Self {
        Self {
            val: inner,
            ph: PhantomData,
            alloc: GlobalAlloc,
        }
    }

    #[inline]
    unsafe fn from_ptr(ptr: *mut RcBox<Type>) -> Self {
        Self {
            val: NonNull::new_unchecked(ptr),
            ph: PhantomData,
            alloc: GlobalAlloc,
        }
    }
}

impl<Type> Rc<Type> {
    pub fn new(value: Type) -> Self {
        let inner = RcBox {
            strong_count: Box::new(0).leak().into(),
            weak_count: Box::new(0).leak().into(),
            value,
        };

        unsafe { Rc::from_inner(Box::new(inner).leak().into()) }
    }

    //pub fn new_cyclic<Function>(data_fn: Function) -> Rc<Type>
    //where
    //    Function: FnOnce(&Weak<Type>) -> Type,
    //{
    //    todo!("New Cyclic is not defined yet, you can help by defining it!")
    //}

    pub fn new_uninit() -> Rc<MaybeUninit<Type>> {
        Rc::new(MaybeUninit::uninit())
    }

    pub fn new_zeroed() -> Rc<MaybeUninit<Type>> {
        Rc::new(MaybeUninit::zeroed())
    }
}

impl<Type, Alloc: AllocatorAPI> Rc<Type, Alloc> {
    pub fn allocator(&self) -> &Alloc {
        &self.alloc
    }

    // TODO: The rest of different allocs
}
