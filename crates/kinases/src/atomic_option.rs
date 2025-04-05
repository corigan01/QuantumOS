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

extern crate alloc;
use alloc::sync::Arc;
use core::{
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::Deref,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

/// An `Atomic<Option<Arc<T>>>`
pub struct AtomicOption<T> {
    guard: AtomicUsize,
    ptr: AtomicPtr<T>,
    ph: PhantomData<T>,
}

impl<T> AtomicOption<T> {
    pub const fn none() -> Self {
        Self {
            guard: AtomicUsize::new(0),
            ptr: AtomicPtr::new(core::ptr::null_mut()),
            ph: PhantomData,
        }
    }

    /// Create a new OptionAtomicArc
    pub fn new(value: Option<Arc<T>>) -> Self {
        let ptr = match value {
            Some(value) => Arc::into_raw(value).cast_mut(),
            None => core::ptr::null_mut(),
        };

        Self {
            ptr: AtomicPtr::new(ptr),
            ph: PhantomData,
            guard: AtomicUsize::new(0),
        }
    }

    fn inc_fake_strong(&self) {
        self.guard.fetch_add(1, Ordering::Acquire);
    }

    fn upgrade(&self, arc: *const T) -> Option<Arc<T>> {
        let mut old = self.guard.load(Ordering::Relaxed);

        loop {
            // If someone else already inc the actual value's strong count, then we don't need to do anything!
            if old == 0 {
                break if arc.is_null() {
                    None
                } else {
                    let inner_arc =
                        unsafe { ManuallyDrop::new(Arc::from_raw(arc)).deref().clone() };
                    assert_ne!(Arc::strong_count(&inner_arc), 0);

                    Some(inner_arc)
                };
            }

            // Otherwise we will try to be the one that inc
            match self.guard.compare_exchange_weak(
                old,
                old - 1,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) if !arc.is_null() => {
                    let inner_arc = unsafe { Arc::from_raw(arc) };
                    assert_ne!(Arc::strong_count(&inner_arc), 0);
                    break Some(inner_arc);
                }
                Ok(_) => break None,
                Err(prev) => {
                    old = prev;
                }
            }
        }
    }

    /// Returns a cloned Arc
    pub fn load(&self) -> Option<Arc<T>> {
        // This will prevent others from dropping the inner arc.
        self.inc_fake_strong();

        let arc_ptr = self.ptr.load(Ordering::Acquire);

        if !arc_ptr.is_null() {
            unsafe { Arc::increment_strong_count(arc_ptr) };
        }

        self.upgrade(arc_ptr)
    }

    pub fn swap(&self, new: Option<Arc<T>>) -> Option<Arc<T>> {
        let new_raw = match new {
            Some(new) => Arc::into_raw(new).cast_mut(),
            None => core::ptr::null_mut(),
        };
        let prev_arc = self.ptr.swap(new_raw, Ordering::SeqCst);

        // This is required because we are no longer holding the prev arc in our structure, so we shouldn't have
        // a strong count for it.
        let upgraded_prev = self.upgrade(prev_arc);
        unsafe { Arc::decrement_strong_count(prev_arc) };

        upgraded_prev
    }
}

impl<T> Drop for AtomicOption<T> {
    fn drop(&mut self) {
        unsafe { Arc::from_raw(self.ptr.load(Ordering::SeqCst)) };
    }
}

impl<T> Clone for AtomicOption<T> {
    fn clone(&self) -> Self {
        Self::new(self.load())
    }
}
