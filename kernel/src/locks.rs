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

use crate::process::thread::RefThread;
use core::fmt::Debug;

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
pub struct InformedScheduleLock(usize);

impl InformedScheduleLock {
    /// Aquire a lock from the scheduler
    pub fn scheduler_notice_locking(encouragement: LockEncouragement, thread: RefThread) -> Self {
        todo!()
    }

    /// release a lock from the scheduler
    pub fn scheduler_notice_unlocking(self) {
        drop(self);
    }
}

impl Drop for InformedScheduleLock {
    fn drop(&mut self) {
        todo!()
    }
}
