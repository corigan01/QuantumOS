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

use alloc::{
    collections::btree_map::BTreeMap,
    string::String,
    sync::{Arc, Weak},
};
use mem::vm::VmProcess;
use thread::{ThreadId, WeakThread};

use crate::locks::RwYieldLock;

pub mod scheduler;
pub mod task;
pub mod thread;

type ProcessId = usize;
pub type RefProcess = Arc<Process>;
pub type WeakProcess = Weak<Process>;

/// A complete execution unit, memory map, threads, etc...
#[derive(Debug)]
pub struct Process {
    /// The unique id of this process
    id: ProcessId,
    /// The name of this process
    name: String,
    /// Weak references to all threads within this process.
    ///
    /// Threads carry strong references to their process, and are the actual scheduling artifacts.
    threads: RwYieldLock<BTreeMap<ThreadId, WeakThread>>,
    /// The memory map of this process
    vm: RwYieldLock<VmProcess>,
}
