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

pub use critical_lock::*;
pub use schedule_lock::*;
pub use thread_cell::*;
pub use yield_lock::*;

mod critical_lock;
mod schedule_lock;
mod thread_cell;
mod yield_lock;

use core::fmt::Debug;

use crate::process::scheduler::Scheduler;

/// A notice to the scheduler about how important this lock might be
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LockEncouragement {
    /// This operation is not of great importance, and other operations are allowed to complete first
    ///
    /// # Note
    /// This operation should be used when updating non-important fields of a task, or for operations
    /// that require long execution times.
    ///
    /// # Effect
    /// This adds no temporary ticks to the current task.
    Weak,
    /// This operation is important to some functioning, and should be advised to complete first
    ///
    /// # Note
    /// This should only be used for operations necessary for some task to be scheduled, as long
    /// operations can starve other tasks of execution time.
    ///
    /// However, unlike `Strong` this operation can be used for tasks that require a bit more time.
    ///
    /// # Effect
    /// This adds a temporary `10` ticks to the current task (~10ms).
    Moderate,
    /// This operation is critically important, and should be strongly advised to complete first
    ///
    /// # Note
    /// This should only be used for small and fast operations, as long operations can starve other
    /// tasks of execution time.
    ///
    /// # Effect
    /// This adds a temporary `20` ticks to the current task (~20ms).
    Strong,
}

/// A lock id used to inform the scheduler when locks are finished
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AcquiredLock(pub usize);

impl AcquiredLock {
    /// Inform the scheduler that we are acquiring an exclusive lock
    ///
    /// This type of lock is used for `Mutexes` or write operations on `RwLocks`. This exclusivly
    /// acquires the lock, such that no other locks can be held before or after the duration of this
    /// `AquiredLock`.
    ///
    /// # Note
    /// This function preforms the lock operation, and always returns when the lock has been aquired.
    pub fn lock_exclusive(lock_id: &LockId, encouragement: LockEncouragement) -> Self {
        Scheduler::acquiredlock_exclusive(lock_id, encouragement)
    }

    /// Inform the scheduler that we are aquiring a shared lock
    ///
    /// This type of lock is used for read operations on `RwLocks`. This 'share'
    /// aquires the lock, such that other 'shared' locks can aquire without blocking.
    /// `AquiredLock`.
    ///
    /// # Note
    /// This function preforms the lock operation, and always returns when the lock has been aquired.
    pub fn lock_shared(lock_id: &LockId, encouragement: LockEncouragement) -> Self {
        Scheduler::acquiredlock_shared(lock_id, encouragement)
    }

    /// Same as `lock_exclusive` except doesn't block.
    pub fn try_lock_exclusive(lock_id: &LockId, encouragement: LockEncouragement) -> Option<Self> {
        Scheduler::get()
            .try_acquiredlock_exclusive(lock_id, encouragement)
            .ok()
    }

    /// Same as `lock_shared` except doesn't block.
    pub fn try_lock_shared(lock_id: &LockId, encouragement: LockEncouragement) -> Option<Self> {
        Scheduler::get()
            .try_acquiredlock_shared(lock_id, encouragement)
            .ok()
    }

    /// Release a lock from the scheduler
    pub fn unlock(self) {
        drop(self);
    }

    /// Force unlock from the scheduler
    pub unsafe fn force_unlock(&mut self) {
        Scheduler::get().aquiredlock_unlock(self);
        self.0 = usize::MAX;
    }
}

impl Drop for AcquiredLock {
    fn drop(&mut self) {
        if self.0 == usize::MAX {
            return;
        }

        Scheduler::get().aquiredlock_unlock(self);
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LockId(pub usize);

impl LockId {
    /// Used to create an ID for a new lock. Used to prevent deadlocks for threads.
    pub fn new() -> Self {
        Scheduler::get().alloc_new_lockid()
    }

    /// Outstanding 'shared' locks
    pub fn current_shared_locks(&self) -> usize {
        Scheduler::get().lockid_total_shared(self)
    }

    /// Outstanding 'exclusive' locks
    pub fn current_exclusive_locks(&self) -> usize {
        Scheduler::get().lockid_total_exclusive(self)
    }
}

impl Drop for LockId {
    fn drop(&mut self) {
        Scheduler::get().dealloc_lockid(self);
    }
}
