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

#![no_std]

use core::sync::atomic::AtomicUsize;

use alloc::{
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    sync::Arc,
};
use runner::{RefRunner, RunnerId};
use sync::spin::mutex::SpinMutex;
use task::{RefTask, TaskId};
extern crate alloc;

pub mod runner;
pub mod task;
pub mod wake;

pub type RefRuntime = Arc<Runtime>;

pub struct Runtime {
    next_runner_id: AtomicUsize,
    next_task_id: AtomicUsize,
    runners: SpinMutex<BTreeMap<RunnerId, RefRunner>>,
    tasks: SpinMutex<BTreeMap<TaskId, RefTask>>,
    needs_runner: SpinMutex<VecDeque<RefTask>>,
}

impl Runtime {
    pub fn new() -> RefRuntime {
        let raw_runtime = Self {
            next_runner_id: AtomicUsize::new(0),
            next_task_id: AtomicUsize::new(0),
            runners: SpinMutex::new(BTreeMap::new()),
            tasks: SpinMutex::new(BTreeMap::new()),
            needs_runner: SpinMutex::new(VecDeque::new()),
        };

        Arc::new(raw_runtime)
    }
}
