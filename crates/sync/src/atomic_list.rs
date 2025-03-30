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

use alloc::boxed::Box;
use core::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

pub struct AtomicLinkedList<T> {
    node: AtomicPtr<AtomicNode<T>>,
}

pub struct AtomicNode<T> {
    next: AtomicPtr<Self>,
    data: T,
}

unsafe impl<T: Send> Send for AtomicLinkedList<T> {}
unsafe impl<T: Send + Sync> Sync for AtomicLinkedList<T> {}
unsafe impl<T: Send> Send for AtomicNode<T> {}
unsafe impl<T: Send + Sync> Sync for AtomicNode<T> {}

impl<T> AtomicLinkedList<T> {
    pub const fn new() -> Self {
        Self {
            node: AtomicPtr::new(null_mut()),
        }
    }

    pub fn push_front(&mut self, value: T) {
        let mut current = self.node.load(Ordering::Relaxed);
        let new = AtomicNode::new_packed(value, current);

        while let Err(failed) =
            self.node
                .compare_exchange(current, new, Ordering::SeqCst, Ordering::Relaxed)
        {
            current = failed;
            unsafe { new.as_mut().unwrap().next = AtomicPtr::new(current) };
        }
    }

    pub fn push_back(&mut self) {}
}

impl<T> AtomicNode<T> {
    pub fn new_packed(data: T, next: *mut Self) -> *mut Self {
        let ptr = Box::into_raw(Box::new(Self {
            next: AtomicPtr::new(next),
            data,
        }));
        ptr
    }

    pub fn push_after(&mut self, value: T) -> AtomicPtr<Self> {
        todo!()
    }
}
