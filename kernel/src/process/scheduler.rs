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
use crate::locks::{InformedScheduleLock, ScheduleLock};
use alloc::{
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    sync::{Arc, Weak},
    vec::Vec,
};
use boolvec::BoolVec;
use lldebug::logln;

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

#[derive(Debug)]
struct AllHoldingLocks {
    ids: BoolVec,
    graph: BTreeMap<InformedScheduleLock, Vec<InformedScheduleLock>>,
    owner: BTreeMap<InformedScheduleLock, WeakThread>,
}

impl AllHoldingLocks {
    pub const fn new() -> Self {
        Self {
            ids: BoolVec::new(),
            graph: BTreeMap::new(),
            owner: BTreeMap::new(),
        }
    }
}

pub type RefScheduler = Arc<Scheduler>;
pub type WeakScheduler = Weak<Scheduler>;

static THE_SCHEDULER: ScheduleLock<Option<RefScheduler>> = ScheduleLock::new(None);

#[derive(Debug)]
pub struct Scheduler {
    /// Weak references to all processes.
    ///
    /// Each Process contains a strong reference to the scheduler, and the scheduler only needs to know
    /// the processes exist.
    process_list: ScheduleLock<BTreeMap<ProcessId, WeakProcess>>,
    /// Weak references to queued threads
    picking_queue: ScheduleLock<VecDeque<ScheduleItem>>,
    /// The currently running thread
    running: ScheduleLock<Option<RefThread>>,
    /// The currently held locks for processes and threads
    held_locks: ScheduleLock<AllHoldingLocks>,
}

impl Scheduler {
    /// Get 'THE' scheduler
    ///
    /// If the scheduler has not be created, this function will create it.
    pub fn get() -> RefScheduler {
        if let Some(sch) = THE_SCHEDULER.lock().clone() {
            return sch;
        } else {
            let mut guard = THE_SCHEDULER.lock();
            logln!("Scheduler Init...");
            let new_scheduler = Arc::new(Self {
                process_list: ScheduleLock::new(BTreeMap::new()),
                picking_queue: ScheduleLock::new(VecDeque::new()),
                running: ScheduleLock::new(None),
                held_locks: ScheduleLock::new(AllHoldingLocks::new()),
            });

            *guard = Some(new_scheduler.clone());
            new_scheduler
        }
    }
}
