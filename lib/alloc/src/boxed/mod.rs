/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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


use core::fmt::{Debug, Display, Formatter};
use core::{borrow, mem};
use core::marker::Unsize;
use core::mem::MaybeUninit;
use core::ops::{CoerceUnsized, Deref, DerefMut, DispatchFromDyn, Receiver};
use core::ptr::NonNull;
use crate::AllocErr;
use crate::heap::{AllocatorAPI, GlobalAlloc};
use crate::memory_layout::MemoryLayout;

pub struct Box<Type: ?Sized, Alloc: AllocatorAPI = GlobalAlloc>(NonNull<Type>, Alloc);

impl<Type> Box<Type> {
    #[must_use]
    pub fn new(value: Type) -> Self {
        Self::new_in(GlobalAlloc, value)
    }

    #[must_use]
    pub fn new_uninit() -> Box<MaybeUninit<Type>> {
        Self::new_uninit_in(GlobalAlloc)
    }

    #[must_use]
    pub fn new_zeroed() -> Box<MaybeUninit<Type>> {
        Self::new_zeroed_in(GlobalAlloc)
    }
}

impl<Type, Alloc: AllocatorAPI> Box<Type, Alloc> {
    pub fn leak<'a>(self) -> &'a mut Type
        where Type: 'a {
        unsafe { mem::ManuallyDrop::new(self).0.as_mut() }
    }

    pub fn try_new_uninit_in_impl(alloc: Alloc, zero: bool) -> Result<Box<MaybeUninit<Type>, Alloc>, AllocErr>
        where Alloc: AllocatorAPI {
        let layout = MemoryLayout::from_type::<Type>();
        let allocation = unsafe { if zero {
            Alloc::allocate_zero(layout)
        } else {
            Alloc::allocate(layout)
        }}?;
        let ptr = NonNull::new(allocation.ptr as *mut MaybeUninit<Type>)
            .ok_or(AllocErr::InternalErr)?;

        Ok(Box(ptr, alloc))
    }

    fn handle_alloc_err<T>(value: Result<Box<T, Alloc>, AllocErr>) -> Box<T, Alloc> {
        value.expect("Could not allocate Box")
    }

    pub fn try_new_zeroed_in(alloc: Alloc) -> Result<Box<MaybeUninit<Type>, Alloc>, AllocErr> {
        Self::try_new_uninit_in_impl(alloc, true)
    }

    pub fn try_new_uninit_in(alloc: Alloc) -> Result<Box<MaybeUninit<Type>, Alloc>, AllocErr>{
        Self::try_new_uninit_in_impl(alloc, false)
    }

    pub fn try_new_in(alloc: Alloc, value: Type) -> Result<Box<Type, Alloc>, AllocErr> {
        let uninit = Self::try_new_uninit_in(alloc)?;
        Ok(uninit.write(value))
    }

    pub fn new_uninit_in(alloc: Alloc) -> Box<MaybeUninit<Type>, Alloc> {
        Self::handle_alloc_err(Self::try_new_uninit_in(alloc))
    }

    pub fn new_zeroed_in(alloc: Alloc) -> Box<MaybeUninit<Type>, Alloc> {
        Self::handle_alloc_err(Self::try_new_zeroed_in(alloc))
    }

    pub fn new_in(alloc: Alloc, value: Type) -> Box<Type, Alloc> {
        Self::handle_alloc_err(Self::try_new_in(alloc, value))
    }

    pub fn into_raw_with_alloc(self) -> (*mut Type, Alloc) {
        (self.leak() as *mut Type, unsafe { MaybeUninit::uninit().assume_init() })
    }

    pub unsafe fn from_raw_in(ptr: *mut Type, alloc: Alloc) -> Self {
        Self(NonNull::new_unchecked(ptr), alloc)
    }

}

impl <Type: Default> Default for Box<Type> {
    fn default() -> Self {
        Box::new(Type::default())
    }
}

impl<Type: Clone, Alloc: AllocatorAPI + Clone> Clone for Box<Type, Alloc> {
    fn clone(&self) -> Self {
        let new_box = Box::new_uninit_in(self.1.clone());
        new_box.write(unsafe { self.0.as_ref().clone() })
    }

    fn clone_from(&mut self, source: &Self) {
        unsafe { self.0.as_ptr().write(source.0.as_ref().clone()) }
    }
}

impl<Type, Alloc: AllocatorAPI> Box<MaybeUninit<Type>, Alloc> {
    pub unsafe fn assume_init(self) -> Box<Type, Alloc> {
        let (ptr, alloc) = self.into_raw_with_alloc();
        Box::from_raw_in(ptr as *mut Type, alloc)
    }

    pub fn write(self, value: Type) -> Box<Type, Alloc> {
        unsafe {
            (self.0.as_ptr() as *mut Type).write(value);
            self.assume_init()
        }
    }
}

impl<Type: ?Sized, Alloc: AllocatorAPI> Receiver for Box<Type, Alloc> {}
impl<T: ?Sized + Unsize<U>, U: ?Sized, A: AllocatorAPI> CoerceUnsized<Box<U, A>> for Box<T, A> {}
impl<T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<Box<U>> for Box<T, GlobalAlloc> {}

impl<Type: ?Sized, Alloc: AllocatorAPI> borrow::Borrow<Type> for Box<Type, Alloc> {
    fn borrow(&self) -> &Type {
        unsafe { self.0.as_ref() }
    }
}

impl<Type: ?Sized, Alloc: AllocatorAPI> borrow::BorrowMut<Type> for Box<Type, Alloc> {
    fn borrow_mut(&mut self) -> &mut Type {
        unsafe { self.0.as_mut() }
    }
}

impl<Type: ?Sized, Alloc: AllocatorAPI> AsRef<Type> for Box<Type, Alloc> {
    fn as_ref(&self) -> &Type {
        unsafe { self.0.as_ref() }
    }
}

impl<Type: ?Sized, Alloc: AllocatorAPI> AsMut<Type> for Box<Type, Alloc> {
    fn as_mut(&mut self) -> &mut Type {
        unsafe { self.0.as_mut() }
    }
}

impl<Type: Display, Alloc: AllocatorAPI> Display for Box<Type, Alloc> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(unsafe { self.0.as_ref() }, f)
    }
}

impl<Type: Debug, Alloc: AllocatorAPI> Debug for Box<Type, Alloc> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(unsafe { self.0.as_ref() }, f)
    }
}

impl<Type: ?Sized, Alloc: AllocatorAPI> Deref for Box<Type, Alloc> {
    type Target = Type;
    
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<Type: ?Sized, Alloc: AllocatorAPI> DerefMut for Box<Type, Alloc> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl<Type: ?Sized, Alloc: AllocatorAPI> Drop for Box<Type, Alloc> {
    fn drop(&mut self) {
        unsafe { Alloc::free(NonNull::<u8>::new(self.0.as_ptr() as *mut u8).unwrap()) }
            .expect("Could not free Box");
    }
}

