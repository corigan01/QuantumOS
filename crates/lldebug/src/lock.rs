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

use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct DebugMutex<T> {
    lock: AtomicBool,
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for DebugMutex<T> {}
unsafe impl<T> Send for DebugMutex<T> {}

pub struct DebugMutexGuard<'a, T> {
    lock: &'a AtomicBool,
    ptr: *mut T,
}

impl<T> DebugMutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            inner: UnsafeCell::new(value),
        }
    }

    pub fn try_lock<'a>(&'a self) -> Option<DebugMutexGuard<'a, T>> {
        while let Err(previous_value) =
            self.lock
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        {
            // Lock failed
            if previous_value {
                return None;
            }
        }

        // here we are locked
        Some(DebugMutexGuard {
            lock: &self.lock,
            ptr: self.inner.get(),
        })
    }
}

impl<'a, T> Deref for DebugMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<'a, T> DerefMut for DebugMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl<'a, T> Drop for DebugMutexGuard<'a, T> {
    fn drop(&mut self) {
        while let Err(previous_value) =
            self.lock
                .compare_exchange(true, false, Ordering::Release, Ordering::Relaxed)
        {
            assert_ne!(
                previous_value, false,
                "Cannot unlock an already unlocked lock"
            );
        }
    }
}
