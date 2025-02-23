/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

use crate::{sync_disable, sync_enable};
use core::{
    cell::UnsafeCell,
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
};

/// A mutex that locks by calling a critical section
pub struct DebugMutex<T: ?Sized> {
    lock: AtomicBool,
    inner: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Sync> Sync for DebugMutex<T> {}
unsafe impl<T: ?Sized + Send + Sync> Send for DebugMutex<T> {}

impl<T> DebugMutex<T> {
    /// Create a new debug Mutex
    pub const fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
            lock: AtomicBool::new(false),
        }
    }
}

impl<T: ?Sized> DebugMutex<T> {
    /// Aquire a lock to the data
    pub fn lock(&self) -> DebugMutexGuard<T> {
        sync_enable();

        if self.lock.load(Ordering::Acquire) {
            panic!("Cannot lock the Mutex multiple times!");
        }

        self.lock.store(true, Ordering::Relaxed);
        DebugMutexGuard {
            ph: PhantomData,
            ptr: NonNull::new(self.inner.get()).unwrap(),
            atomic: &self.lock,
        }
    }

    /// Get a mutable ptr to the inner data
    ///
    /// # Safety
    /// There must be no other locks to this data.
    pub unsafe fn as_mut_ptr(&self) -> *mut T {
        self.inner.get()
    }
}

impl<T: ?Sized + Debug> Debug for DebugMutex<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.lock().fmt(f)
    }
}

impl<T: ?Sized + Display> Display for DebugMutex<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.lock().fmt(f)
    }
}

/// A protected lock to the data
pub struct DebugMutexGuard<'a, T: ?Sized> {
    ph: PhantomData<&'a ()>,
    ptr: NonNull<T>,
    atomic: &'a AtomicBool,
}

unsafe impl<'a, T: ?Sized + Sync> Sync for DebugMutexGuard<'a, T> {}
unsafe impl<'a, T: ?Sized + Send + Sync> Send for DebugMutexGuard<'a, T> {}

impl<'a, T: ?Sized> Deref for DebugMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<'a, T: ?Sized> DerefMut for DebugMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<'a, T: ?Sized> Drop for DebugMutexGuard<'a, T> {
    fn drop(&mut self) {
        if !self.atomic.swap(false, Ordering::Release) {
            panic!("Cannot release a lock that was never locked!");
        }

        sync_disable();
    }
}

impl<'a, T: ?Sized + Debug> Debug for DebugMutexGuard<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe { self.ptr.as_ref().fmt(f) }
    }
}

impl<'a, T: ?Sized + Display> Display for DebugMutexGuard<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe { self.ptr.as_ref().fmt(f) }
    }
}

impl<'a, T: ?Sized> DebugMutexGuard<'a, T> {
    /// Release the inner lock and get the inner value
    pub unsafe fn release_lock(mut self) -> &'a mut T {
        unsafe { self.ptr.as_mut() }
    }
}
