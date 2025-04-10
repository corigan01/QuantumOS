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

extern crate alloc;

use crate::spin::mutex::SpinMutex;
use alloc::collections::VecDeque;
use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
    task::Waker,
};

/// An atomic-ish optional waker.
///
/// A `WakeCell` can either contain exactly *one* waker, or none at all. All modifications
/// internally are atomic, and could be assumed to hold a `SpinMutex<Option<Waker>>`.
///
/// # Speed
/// `WakeCell` internally uses 'Compare-Exchange' loops and thus in some instances
/// can be undesirable as essentially it's a spinlock. Operations should only block
/// for the duration of one waker write, however, thread scheduling could result in
/// long wait times which eat large amounts of CPU time.
///
/// # Safety
/// This `waker` must remain valid for the duration it is used within this wake cell,
/// and must point to `'static` lifetime memory. Useal `Waker` safety applies.
pub struct WakeCell {
    lock: AtomicUsize,
    waker: UnsafeCell<MaybeUninit<Waker>>,
}

impl WakeCell {
    const NONE: usize = 0;
    const SOME: usize = 1 << 0;
    const LOCKING: usize = 1 << 1;

    /// Create a new empty `WakeCell` that internally contains `None` for its waker.
    pub const fn new() -> Self {
        Self {
            lock: AtomicUsize::new(Self::NONE),
            waker: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Write this `waker` to the cell returning the previous waker if one exists.
    ///
    /// # Speed
    /// `WakeCell` internally uses 'Compare-Exchange' loops and thus in some instances
    /// can be undesirable as essentially it's a spinlock. Operations should only block
    /// for the duration of one waker write, however, thread scheduling could result in
    /// long wait times which eat large amounts of CPU time.
    ///
    /// # Safety
    /// This `waker` must remain valid for the duration it is used within this wake cell,
    /// and must point to `'static` lifetime memory. Useal `Waker` safety applies.
    pub fn attach(&self, waker: Waker) -> Option<Waker> {
        let mut current = self.lock.load(Ordering::Relaxed);

        // Aquire the `Some` lock
        while let Err(failed) = self.lock.compare_exchange_weak(
            current & !Self::LOCKING,
            Self::SOME | Self::LOCKING,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            current = failed;
        }

        // If `Some(...)` is contained previously
        let old_waker = if current & Self::SOME != 0 {
            Some(unsafe { (*self.waker.get()).assume_init_read() })
        } else {
            None
        };

        // Write our value into our storage
        unsafe { (*self.waker.get()).write(waker) };

        // Unlock
        self.lock.store(Self::SOME, Ordering::SeqCst);

        old_waker
    }

    /// Takes the waker from the `WakeCell` and returns it if one exists, putting `None` in its place.
    ///
    /// # Speed
    /// `WakeCell` internally uses 'Compare-Exchange' loops and thus in some instances
    /// can be undesirable as essentially it's a spinlock. Operations should only block
    /// for the duration of one waker write, however, thread scheduling could result in
    /// long wait times which eat large amounts of CPU time.
    ///
    /// # Safety
    /// This `waker` must remain valid for the duration it is used within this wake cell,
    /// and must point to `'static` lifetime memory. Useal `Waker` safety applies.
    pub fn take_waker(&self) -> Option<Waker> {
        let mut current = self.lock.load(Ordering::Relaxed);

        // Optimization: If no waker exists, we don't need to continue trying to aquire
        // the lock. We can simply return None!
        if current & Self::SOME == 0 {
            return None;
        }

        // Aquire the `None` lock
        while let Err(failed) = self.lock.compare_exchange_weak(
            (current & !Self::LOCKING) | Self::SOME,
            Self::NONE | Self::LOCKING,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            // We need to do it here too to ensure it never goes None during the aquire.
            if failed & Self::SOME == 0 {
                return None;
            }

            current = failed;
        }

        // If `Some(...)` is contained previously
        let old_waker = if current & Self::SOME != 0 {
            Some(unsafe { (*self.waker.get()).assume_init_read() })
        } else {
            None
        };

        // Unlock
        self.lock.store(Self::NONE, Ordering::SeqCst);

        old_waker
    }

    /// Directly call the contained waker (if exists) replacing it with `None`.
    ///
    /// This function calls `take_waker` internally, and thus has the same behavior
    /// as:
    /// ```rust
    /// use kinases::wake::WakeCell;
    ///
    /// let waker = WakeCell::new();
    /// if let Some(contained_waker) = self.take_waker() {
    ///    contained_waker.wake();
    /// }
    /// ```
    pub fn wake(&self) {
        if let Some(waker) = self.take_waker() {
            waker.wake();
        }
    }

    /// Drop the contained waker (if exists) replacing it with `None`.
    ///
    /// This function calls `take_waker` internally, and thus has the same behavior
    /// as:
    /// ```rust
    /// use kinases::wake::WakeCell;
    ///
    /// let waker = WakeCell::new();
    /// let _ = self.take_waker();
    /// ```
    pub fn empty(&self) {
        _ = self.take_waker();
    }
}

pub struct WakeQueue {
    // TODO: Upgrade to a non-blocking queue in the future
    queue: SpinMutex<VecDeque<Waker>>,
}

impl WakeQueue {
    pub const fn new() -> Self {
        Self {
            queue: SpinMutex::new(VecDeque::new()),
        }
    }
}
