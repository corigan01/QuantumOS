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

use crate::atomic_option::AtomicOption;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct AtomicNode<T> {
    state: AtomicUsize,
    next: AtomicOption<AtomicNode<T>>,
    prev: AtomicOption<AtomicNode<T>>,
    value: T,
}

impl<T> AtomicNode<T> {
    const ALIVE: usize = 1;
    const DEAD: usize = 0;

    fn new(next: Option<Arc<Self>>, prev: Option<Arc<Self>>, value: T) -> Arc<Self> {
        Arc::new(Self {
            state: AtomicUsize::new(Self::ALIVE),
            next: AtomicOption::new(next),
            prev: AtomicOption::new(prev),
            value,
        })
    }

    fn insert_between(left: Option<Arc<Self>>, new: T, right: Option<Arc<Self>>) {
        let new_node = Self::new(right.clone(), left.clone(), new);

        if let Some(left) = left {
            // check both before and after trying to swap if the node is dead
            //
            // if the node is dead, restore the previous value
            left.next.swap(Some(new_node.clone()));
        }
        if let Some(right) = right {
            right.prev.swap(Some(new_node.clone()));
        }
    }

    pub fn push_left(self: Arc<Self>, value: T) {
        let left_node = self.prev.load();
        let right_node = Some(self);

        Self::insert_between(left_node, value, right_node);
    }

    pub fn push_right(self: Arc<Self>, value: T) {
        let right_node = self.next.load();
        let left_node = Some(self);

        Self::insert_between(left_node, value, right_node);
    }

    pub fn unlink(self: Arc<Self>) {
        // If we fail to set the marker, we are already being unlinked
        if let Err(_) = self.state.compare_exchange(
            Self::ALIVE,
            Self::DEAD,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            return;
        }

        let old_prev = self.prev.load();
        let old_next = self.next.load();

        if let Some(prev) = old_prev.clone() {
            prev.next.swap(old_next.clone());
        }
        if let Some(next) = old_next {
            next.prev.swap(old_prev.clone());
        }
    }
}
