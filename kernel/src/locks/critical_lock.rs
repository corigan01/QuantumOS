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

use super::{AcquiredLock, LockEncouragement, LockId};
use core::cell::UnsafeCell;
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};

/// A `RwLock` with relax behavior that yields until its lock is ready
pub struct RwCriticalLock<T: ?Sized> {
    lock: LockId,
    inner: UnsafeCell<T>,
}

impl<T> RwCriticalLock<T> {
    /// Create a new read write yield lock
    pub fn new(value: T) -> Self {
        Self {
            lock: LockId::new(),
            inner: UnsafeCell::new(value),
        }
    }
}

// I want to make this lock ensure that it cannot deadlock. For example, if the same thread
// tries to aquire a write lock while holding a read lock, the kernel should panic!

impl<T: ?Sized> RwCriticalLock<T> {
    /// Aquire a read lock to the `RwCriticalLock`
    pub fn read<'a>(&'a self, p: LockEncouragement) -> ReadCriticalLockGuard<'a, T> {
        let lock = AcquiredLock::lock_shared(&self.lock, p);

        ReadCriticalLockGuard {
            lock_id: &self.lock,
            lock,
            ptr: self.inner.get(),
        }
    }

    /// Aquire a write lock to the `RwCriticalLock`
    pub fn write<'a>(&'a self) -> WriteCriticalLockGuard<'a, T> {
        let lock = AcquiredLock::lock_exclusive(&self.lock, LockEncouragement::Strong);

        WriteCriticalLockGuard {
            lock_id: &self.lock,
            lock,
            ptr: self.inner.get(),
        }
    }

    /// Try to Aquire a read lock to the `RwCriticalLock`
    pub fn try_read<'a>(&'a self, p: LockEncouragement) -> Option<ReadCriticalLockGuard<'a, T>> {
        let lock = AcquiredLock::try_lock_shared(&self.lock, p)?;

        Some(ReadCriticalLockGuard {
            lock_id: &self.lock,
            lock,
            ptr: self.inner.get(),
        })
    }

    /// Try to Aquire a write lock to the `RwCriticalLock`
    pub fn try_write<'a>(&'a self, p: LockEncouragement) -> Option<WriteCriticalLockGuard<'a, T>> {
        let lock = AcquiredLock::try_lock_exclusive(&self.lock, p)?;

        Some(WriteCriticalLockGuard {
            lock_id: &self.lock,
            lock,
            ptr: self.inner.get(),
        })
    }
}

impl<T: ?Sized + Debug> Debug for RwCriticalLock<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut s = f.debug_struct("RwCriticalLock");
        s.field("readers", &self.lock.current_shared_locks())
            .field("writers", &self.lock.current_exclusive_locks());

        if let Some(lock) = self.try_read(LockEncouragement::Weak) {
            s.field("inner", &lock).finish()
        } else {
            s.field("inner", &"[locked]").finish_non_exhaustive()
        }
    }
}

pub struct ReadCriticalLockGuard<'a, T: ?Sized> {
    lock_id: &'a LockId,
    lock: AcquiredLock,
    ptr: *const T,
}

pub struct WriteCriticalLockGuard<'a, T: ?Sized> {
    lock_id: &'a LockId,
    lock: AcquiredLock,
    ptr: *mut T,
}

impl<'a, T: ?Sized + Debug> Debug for ReadCriticalLockGuard<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe { &*self.ptr }.fmt(f)
    }
}

impl<'a, T: ?Sized + Debug> Debug for WriteCriticalLockGuard<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe { &*self.ptr }.fmt(f)
    }
}

impl<'a, T: ?Sized> Deref for ReadCriticalLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<'a, T: ?Sized> Deref for WriteCriticalLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<'a, T: ?Sized> DerefMut for WriteCriticalLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}
