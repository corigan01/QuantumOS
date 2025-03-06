/*
  ____                 __               __  __
 / __ \__ _____ ____  / /___ ____ _    / / / /__ ___ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /_/ (_-</ -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/  \____/___/\__/_/
  Part of the Quantum OS Kernel

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
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};
use quantum_portal::client::yield_me;

/// A QuantumOS Mutex with yield relax behavior.
pub struct Mutex<T: ?Sized> {
    lock: AtomicBool,
    inner: UnsafeCell<T>,
}

unsafe impl<T: ?Sized> Send for Mutex<T> {}
unsafe impl<T: ?Sized> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Create a new mutex lock with the provided data
    pub const fn new(value: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            inner: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Try to lock this mutex
    pub fn try_lock<'a>(&'a self) -> Option<MutexGuard<'a, T>> {
        while let Err(failed_lock) =
            self.lock
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        {
            // If this lock is already set, we fail the lock
            if failed_lock {
                return None;
            }
        }

        Some(MutexGuard {
            lock: &self.lock,
            ptr: self.inner.get(),
        })
    }

    /// Lock this mutex
    pub fn lock<'a>(&'a self) -> MutexGuard<'a, T> {
        loop {
            match self.try_lock() {
                Some(lock) => return lock,
                None => {
                    // TODO: In the future we should add thread parking to the scheduler
                    yield_me();
                }
            }
        }
    }
}

/// A protected guard for the mutex
pub struct MutexGuard<'a, T: ?Sized> {
    lock: &'a AtomicBool,
    ptr: *mut T,
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        while let Err(failed_unlock) =
            self.lock
                .compare_exchange(true, false, Ordering::Release, Ordering::Relaxed)
        {
            assert!(
                failed_unlock,
                "Mutex is marked as unlocked while dropping MutexGuard"
            );
        }
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl<'a, T: ?Sized + Debug> Debug for MutexGuard<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe { &*self.ptr }.fmt(f)
    }
}

impl<'a, T: ?Sized + Display> Display for MutexGuard<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe { &*self.ptr }.fmt(f)
    }
}

impl<T: Clone> Clone for Mutex<T> {
    fn clone(&self) -> Self {
        Mutex::new(self.lock().clone())
    }
}
