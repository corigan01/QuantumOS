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

use super::{
    ProcessId, WeakProcess,
    thread::{RefThread, WeakThread},
};
use alloc::{
    collections::{binary_heap::BinaryHeap, btree_map::BTreeMap},
    sync::{Arc, Weak},
};

/// A priority queue item with a weak reference to its owned thread
#[derive(Debug)]
struct ScheduleItem {
    priority: isize,
    thread: WeakThread,
}

impl Eq for ScheduleItem {}
impl PartialEq for ScheduleItem {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl PartialOrd for ScheduleItem {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.priority.cmp(&other.priority))
    }
}

impl Ord for ScheduleItem {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub type RefScheduler = Arc<Scheduler>;
pub type WeakScheduler = Weak<Scheduler>;

#[derive(Debug)]
pub struct Scheduler {
    /// Weak references to all processes.
    ///
    /// Each Process contains a strong reference to the scheduler, and the scheduler only needs to know
    /// the processes exist.
    process_list: BTreeMap<ProcessId, WeakProcess>,
    /// Weak references to queued threads
    picking_queue: BinaryHeap<ScheduleItem>,
    /// The currently running thread
    running: Option<RefThread>,
}
