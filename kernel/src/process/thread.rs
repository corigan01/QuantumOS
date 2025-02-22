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

use super::{ProcessEntry, RefProcess, scheduler::Scheduler, task::Task};
use crate::locks::ThreadCell;
use alloc::sync::{Arc, Weak};

pub type ThreadId = usize;
pub type RefThread = Arc<Thread>;
pub type WeakThread = Weak<Thread>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreadContextKind {
    Userspace,
    Kernel,
}

/// A userspace execution unit, like a [`Task`] but for userspace.
#[derive(Debug)]
pub struct Thread {
    /// The thread's id
    pub id: ThreadId,
    /// To which does this thread switches to, Kernel or Userspace
    pub context_kind: ThreadContextKind,
    /// This is the kernel task that starts/resumes the context.
    ///
    /// The context itself is stored within the task's stack, and could be
    /// placed either via an interrupt or via a system call.
    pub task: ThreadCell<Task>,
    /// The parent process that this thread represents
    pub process: RefProcess,
    /// Init Userspace entrypoint
    // TODO: Maybe there could be a better way of passing the `ProcessEntry` into
    // `userspace_thread_begin`?
    userspace_entry: Option<ProcessEntry>,
}

impl Thread {
    /// Create a new userspace thread
    pub fn new_user(process: RefProcess, entry_point: ProcessEntry) -> RefThread {
        let id = process.alloc_thread_id();
        let task = Task::new(userspace_thread_begin);

        let thread = Arc::new(Self {
            id,
            context_kind: ThreadContextKind::Userspace,
            task: ThreadCell::new(task),
            process,
            userspace_entry: Some(entry_point),
        });

        let s = Scheduler::get();
        s.register_new_thread(thread.clone());

        thread
    }

    /// Create a new kernel thread
    pub fn new_kernel(process: RefProcess, entry_point: fn()) -> RefThread {
        let id = process.alloc_thread_id();
        let task = Task::new(entry_point);

        let thread = Arc::new(Self {
            id,
            context_kind: ThreadContextKind::Kernel,
            task: ThreadCell::new(task),
            process,
            userspace_entry: None,
        });

        let s = Scheduler::get();
        s.register_new_thread(thread.clone());

        thread
    }
}

fn userspace_thread_begin() {
    todo!()
}

extern "C" fn asm_syscall_entry() {
    todo!()
}

extern "C" fn asm_syscall_exit() {
    todo!()
}
