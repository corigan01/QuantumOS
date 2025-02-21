/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
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

use core::fmt::Debug;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::{cell::UnsafeCell, sync::atomic::AtomicBool};

static SCHEDULE_LOCKS_HELD: AtomicUsize = AtomicUsize::new(0);

/// Get the current number of active scheduler locks
pub fn current_scheduler_locks() -> usize {
    SCHEDULE_LOCKS_HELD.load(Ordering::Relaxed)
}

/// A `Mutex` with relax behavior that prevents schedules
pub struct ScheduleLock<T: ?Sized> {
    lock: AtomicBool,
    inner: UnsafeCell<T>,
}

impl<T: ?Sized + Debug> Debug for ScheduleLock<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.lock().fmt(f)
    }
}

unsafe impl<T: ?Sized> Send for ScheduleLock<T> {}
unsafe impl<T: ?Sized> Sync for ScheduleLock<T> {}

pub struct ScheduleLockGuard<'a, T: ?Sized> {
    lock: &'a AtomicBool,
    ptr: *mut T,
}

impl<T> ScheduleLock<T> {
    /// Create a new schedule lock
    pub const fn new(value: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            inner: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> ScheduleLock<T> {
    /// Lock the scheduler.
    ///
    /// This does not disable interrupts, but instead just prevents interrupts from causing
    /// the scheduler to switch to another process.
    pub fn lock<'a>(&'a self) -> ScheduleLockGuard<'a, T> {
        // Aquire the lock
        while let Err(previous_value) =
            self.lock
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        {
            if previous_value {
                panic!("Cannot aquire the scheduler lock twice!");
            }
        }

        // Increment the scheduler lock
        SCHEDULE_LOCKS_HELD.fetch_add(1, Ordering::SeqCst);

        ScheduleLockGuard {
            lock: &self.lock,
            ptr: self.inner.get(),
        }
    }
}

impl<'a, T: ?Sized> Deref for ScheduleLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<'a, T: ?Sized> DerefMut for ScheduleLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl<'a, T: ?Sized + Debug> Debug for ScheduleLockGuard<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe { &*self.ptr }.fmt(f)
    }
}

impl<'a, T: ?Sized> Drop for ScheduleLockGuard<'a, T> {
    fn drop(&mut self) {
        // release the lock
        while let Err(previous_value) =
            self.lock
                .compare_exchange(true, false, Ordering::Release, Ordering::Relaxed)
        {
            if !previous_value {
                panic!("Cannot release a lock that was never locked!");
            }
        }

        // decrement the scheduler lock
        SCHEDULE_LOCKS_HELD.fetch_sub(1, Ordering::SeqCst);
    }
}
