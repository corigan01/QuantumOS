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

use crate::locks::RwYieldLock;

use super::{RefProcess, scheduler::RefScheduler, task::Task};
use alloc::sync::{Arc, Weak};

pub type ThreadId = usize;
pub type RefThread = Arc<Thread>;
pub type WeakThread = Weak<Thread>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreadState {
    /// The thread is currently Running
    Running,
    /// The thread is read to be ran, awaiting the run queue
    Runnable,
    /// The thread is in the run queue, awaiting being scheduled
    Picking,
    /// The thread is being blocked by some operation and cannot be scheduled
    Blocking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreadContextKind {
    Userspace,
    Kernel,
}

/// A userspace execution unit, like a [`Task`] but for userspace.
#[derive(Debug)]
pub struct Thread {
    /// The thread's id
    id: ThreadId,
    /// To which does this thread switches to, Kernel or Userspace
    context_kind: ThreadContextKind,
    /// This is the kernel task that starts/resumes the context.
    ///
    /// The context itself is stored within the task's stack, and could be
    /// placed either via an interrupt or via a system call.
    task: Task,
    /// The scheduling state of this thread
    state: RwYieldLock<ThreadState>,
    /// The parent process that this thread represents
    process: RefProcess,
    /// The scheduler who manages this process's threads
    scheduler: RefScheduler,
}
