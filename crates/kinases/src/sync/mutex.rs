/*
   ___   __        _   __
  / _ | / /__  ___| | / /__ _______ _
 / __ |/ / _ \/ -_) |/ / -_) __/ _ `/
/_/ |_/_/\___/\__/|___/\__/_/  \_,_/

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

use super::semaphore::{AcquiredTickets, Semaphore, SemaphoreError};
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

pub struct Mutex<T> {
    semaphore: Semaphore,
    data: UnsafeCell<T>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum MutexError {
    AlreadyLocked,
    Poisoned,
    Closed,
}

impl From<SemaphoreError> for MutexError {
    fn from(value: SemaphoreError) -> Self {
        match value {
            SemaphoreError::WaitingOnEnoughTickets => MutexError::AlreadyLocked,
            SemaphoreError::Poisoned => MutexError::Poisoned,
            SemaphoreError::Closed => MutexError::Closed,
            SemaphoreError::NotEnoughTotalTickets | SemaphoreError::AlreadyWaiting => {
                unreachable!()
            }
        }
    }
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            semaphore: Semaphore::new(1),
            data: UnsafeCell::new(value),
        }
    }

    pub fn try_lock(&self) -> Result<MutexGuard<'_, T>, MutexError> {
        Ok(self
            .semaphore
            .acquire(1)
            .try_acquire()
            .map(|aquired| MutexGuard {
                _acquire: aquired,
                value: self.data.get(),
            })?)
    }

    pub fn blocking_lock(&self) -> MutexGuard<'_, T> {
        let aquired = self
            .semaphore
            .acquire(1)
            .blocking_acquire()
            .expect("Failed to lock Mutex");

        MutexGuard {
            _acquire: aquired,
            value: self.data.get(),
        }
    }

    pub fn blocking_lock_result(&self) -> Result<MutexGuard<'_, T>, MutexError> {
        let aquired = self.semaphore.acquire(1).blocking_acquire()?;

        Ok(MutexGuard {
            _acquire: aquired,
            value: self.data.get(),
        })
    }

    pub async fn lock(&self) -> MutexGuard<'_, T> {
        let aquired = self
            .semaphore
            .acquire(1)
            .await
            .expect("Failed to lock Mutex");

        MutexGuard {
            _acquire: aquired,
            value: self.data.get(),
        }
    }

    pub async fn lock_result(&self) -> Result<MutexGuard<'_, T>, MutexError> {
        Ok(self.semaphore.acquire(1).await.map(|aquired| MutexGuard {
            _acquire: aquired,
            value: self.data.get(),
        })?)
    }
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send + Sync> Sync for Mutex<T> {}

#[must_use = "MutexGuard will release the lock when dropped."]
pub struct MutexGuard<'a, T: ?Sized> {
    _acquire: AcquiredTickets<'a>,
    value: *mut T,
}
unsafe impl<'a, T: ?Sized + Send> Send for MutexGuard<'a, T> {}
unsafe impl<'a, T: ?Sized + Send + Sync> Sync for MutexGuard<'a, T> {}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.value }
    }
}

impl<'a, T: ?Sized + core::fmt::Debug> core::fmt::Debug for MutexGuard<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        T::fmt(self, f)
    }
}

impl<'a, T: ?Sized + core::fmt::Display> core::fmt::Display for MutexGuard<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        T::fmt(self, f)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{sync::Arc, thread, vec::Vec};

    extern crate std;

    #[test]
    fn test_mutex_locks() {
        let m = Arc::new(Mutex::<i32>::new(0));

        #[cfg(not(miri))]
        const MAX_THREADS: usize = 32;
        #[cfg(not(miri))]
        const MAX_THREAD_ITER: usize = 10000;

        // This is used otherwise miri takes forever to run
        #[cfg(miri)]
        const MAX_THREADS: usize = 2;
        #[cfg(miri)]
        const MAX_THREAD_ITER: usize = 100;

        let mut thread_joins = Vec::new();
        for _ in 0..MAX_THREADS {
            let m = m.clone();
            thread_joins.push(thread::spawn(move || {
                // Make sure all the threads stay busy for a bit
                for _ in 0..MAX_THREAD_ITER {
                    let mut value = m.blocking_lock();
                    assert_eq!(*value, 0);

                    *value += 1;
                    assert_eq!(*value, 1);

                    *value = 0;
                    assert_eq!(*value, 0);

                    drop(value);
                }
            }));
        }

        for thread in thread_joins {
            thread.join().unwrap();
        }
    }
}
