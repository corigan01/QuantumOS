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

use core::{
    cell::UnsafeCell,
    fmt::Debug,
    sync::atomic::{AtomicBool, AtomicUsize},
};

/// A notice to the scheduler about how important this lock might be
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encouragement {
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

/// A `RwLock` with relax behavior that yields until its lock is ready
pub struct RwYieldLock<T: ?Sized> {
    lock: AtomicUsize,
    inner: UnsafeCell<T>,
}

impl<T: ?Sized + Debug> Debug for RwYieldLock<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

/// A `Mutex` with relax behavior that prevents schedules
pub struct ScheduleLock<T: ?Sized> {
    lock: AtomicBool,
    inner: UnsafeCell<T>,
}

impl<T: ?Sized + Debug> Debug for ScheduleLock<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

// We need to track how many interrupts happen when we hold a `ScheduleLock` that way
// when we drop the lock, we are able to retry the operations that needed the lock.
//
// Or maybe we could use a Queue of operations or closures maybe? That could be memory
// intensive though? But we need to do something because I don't want to disable interrupts
// ever!
