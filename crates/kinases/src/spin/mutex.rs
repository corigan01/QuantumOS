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

use super::{DefaultSpin, SpinRelax};
use core::{
    cell::UnsafeCell,
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

/// A Spin based Mutex
pub struct SpinMutex<T: ?Sized, R: SpinRelax = DefaultSpin> {
    lock: AtomicUsize,
    ph: PhantomData<R>,
    cell: UnsafeCell<T>,
}

impl<T: Clone, R: SpinRelax> Clone for SpinMutex<T, R> {
    fn clone(&self) -> Self {
        SpinMutex::new(self.lock().deref().clone())
    }
}

unsafe impl<T: ?Sized + Send> Send for SpinMutex<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for SpinMutex<T> {}

impl<T: ?Sized, R: SpinRelax> Drop for SpinMutex<T, R> {
    fn drop(&mut self) {
        // This should be save because if the mutex is being dropped, there should never be any
        // threads aquired.
        assert_eq!(
            self.lock.load(Ordering::Relaxed),
            0,
            "UB: Dropping SpinMutex with aquired lock!"
        );
    }
}

impl<T, R: SpinRelax> SpinMutex<T, R> {
    pub const fn new(value: T) -> Self {
        Self {
            lock: AtomicUsize::new(0),
            ph: PhantomData,
            cell: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized, R: SpinRelax> SpinMutex<T, R> {
    const LOCKED: usize = 1;

    pub fn try_lock<'a>(&'a self) -> Option<SpinMutexGuard<'a, T>> {
        if self.lock.swap(Self::LOCKED, Ordering::Acquire) == Self::LOCKED {
            None
        } else {
            Some(SpinMutexGuard {
                lock: &self.lock,
                cell: self.cell.get(),
            })
        }
    }

    pub fn lock<'a>(&'a self) -> SpinMutexGuard<'a, T> {
        loop {
            match self.try_lock() {
                Some(value) => break value,
                None => {
                    R::back_off();
                }
            }
        }
    }

    pub unsafe fn get(&self) -> *mut T {
        self.cell.get()
    }
}

pub struct SpinMutexGuard<'a, T: ?Sized> {
    lock: &'a AtomicUsize,
    cell: *mut T,
}

impl<'a, T: ?Sized> Drop for SpinMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(0, Ordering::Release);
    }
}

impl<'a, T: ?Sized> Deref for SpinMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.cell }
    }
}

impl<'a, T: ?Sized> DerefMut for SpinMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.cell }
    }
}

impl<'a, T: ?Sized + Debug> Debug for SpinMutexGuard<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe { &*self.cell }.fmt(f)
    }
}

impl<'a, T: ?Sized + Display> Display for SpinMutexGuard<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe { &*self.cell }.fmt(f)
    }
}
