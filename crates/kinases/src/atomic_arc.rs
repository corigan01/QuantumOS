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

/// An atomic, atomic reference counted ptr of `T`.
pub struct AtomicArc<T> {
    guard: AtomicUsize,
    ptr: AtomicPtr<T>,
    ph: PhantomData<T>,
}

impl<T> AtomicArc<T> {
    /// Create a new AtomicArc
    pub fn new(value: Arc<T>) -> Self {
        let begin = Arc::into_raw(value).cast_mut();

        Self {
            ptr: AtomicPtr::new(begin),
            ph: PhantomData,
            guard: AtomicUsize::new(0),
        }
    }

    fn inc_fake_strong(&self) {
        self.guard.fetch_add(1, Ordering::Acquire);
    }

    fn upgrade(&self, arc: *const T) -> Arc<T> {
        let mut old = self.guard.load(Ordering::Relaxed);

        let upgraded_arc = loop {
            // If someone else already inc the actual value's strong count, then we don't need to do anything!
            if old == 0 {
                break unsafe { ManuallyDrop::new(Arc::from_raw(arc)).deref().clone() };
            }

            // Otherwise we will try to be the one that inc
            match self.guard.compare_exchange_weak(
                old,
                old - 1,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    break unsafe { Arc::from_raw(arc) };
                }
                Err(prev) => {
                    old = prev;
                }
            }
        };
        assert_ne!(Arc::strong_count(&upgraded_arc), 0);

        upgraded_arc
    }

    /// Returns a cloned Arc
    pub fn load(&self) -> Arc<T> {
        // This will prevent others from dropping the inner arc.
        self.inc_fake_strong();

        let arc_ptr = self.ptr.load(Ordering::Acquire);
        unsafe { Arc::increment_strong_count(arc_ptr) };

        self.upgrade(arc_ptr)
    }

    pub fn swap(&self, new: Arc<T>) -> Arc<T> {
        let new_raw = Arc::into_raw(new).cast_mut();
        let prev_arc = self.ptr.swap(new_raw, Ordering::SeqCst);

        // This is required because we are no longer holding the prev arc in our structure, so we shouldn't have
        // a strong count for it.
        let upgraded_prev = self.upgrade(prev_arc);
        unsafe { Arc::decrement_strong_count(prev_arc) };

        upgraded_prev
    }
}

impl<T> Drop for AtomicArc<T> {
    fn drop(&mut self) {
        unsafe { Arc::from_raw(self.ptr.load(Ordering::SeqCst)) };
    }
}

impl<T> Clone for AtomicArc<T> {
    fn clone(&self) -> Self {
        Self::new(self.load())
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;

    #[test]
    fn test_single_threaded() {
        let arc = Arc::new(1);
        let atomic_arc = AtomicArc::new(arc.clone());

        assert_eq!(*atomic_arc.load(), 1);
        assert_eq!(Arc::strong_count(&arc), 2);

        assert_eq!(Arc::strong_count(&atomic_arc.load()), 3);
        assert_eq!(Arc::strong_count(&atomic_arc.swap(Arc::new(2))), 2);

        assert_eq!(*atomic_arc.load(), 2);
        assert_eq!(Arc::strong_count(&arc), 1);
        assert_eq!(Arc::strong_count(&atomic_arc.load()), 2);
    }

    #[test]
    fn test_multi_threaded() {
        let atomic_arc = AtomicArc::new(Arc::new(i32::MAX));

        let mut spawned_threads = std::vec::Vec::new();
        for thread_number in 0..10 {
            let atomic_arc = atomic_arc.clone();
            spawned_threads.push(std::thread::spawn(move || {
                let thread_owned = Arc::new(thread_number);
                let prev_arc = atomic_arc.swap(thread_owned.clone());

                assert!(Arc::strong_count(&prev_arc) >= 1);
                assert!(Arc::strong_count(&thread_owned) >= 1);

                atomic_arc.swap(prev_arc);

                assert!(Arc::strong_count(&thread_owned) >= 1);
            }));
        }

        for thread in spawned_threads {
            _ = thread.join();
        }

        assert_eq!(*atomic_arc.load(), i32::MAX);
        assert_eq!(Arc::strong_count(&atomic_arc.load()), 2);
        assert_eq!(Arc::strong_count(&atomic_arc.swap(Arc::new(0))), 1);
    }
}
