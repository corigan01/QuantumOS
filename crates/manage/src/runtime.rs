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

use crate::vtask;
use alloc::boxed::Box;
use core::ops::Deref;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimeStatus {
    Running,
    ShuttingDown,
    Panic,
}

pub trait RuntimeSupport {
    /// The given task has been awoken (called wake) and requests another poll.
    ///
    /// When called, the runtime should provide at least one additional call
    /// to the future's poll method. It is up to the runtime on exactly how this step is accomplished,
    /// but ultimately `Task`'s expect to be polled after calling this function.
    fn task_awoken(&self, task: vtask::AnonTask);

    /// Get the next awaiting task from the runtime.
    ///
    /// This method is not required for all runtimes, but is more of a helper method in multithreaded
    /// runtimes. This function should return the next future that the runtime desires to be polled.
    fn next_awaiting_task(&self) -> Option<GuardedJob> {
        None
    }

    /// Get the current status of the runtime.
    ///
    /// This method is not required for all runtimes, but if implemented should return the current
    /// active status of the runtime. This is to help multithreaded runners
    fn runtime_status(&self) -> RuntimeStatus {
        RuntimeStatus::Running
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GuardedJobStatus {
    Finished,
    Dropped,
    Canceled,
}

pub struct GuardedJob {
    job: vtask::AnonTask,
    callback: Option<Box<dyn FnOnce(GuardedJobStatus, vtask::AnonTask) + Send>>,
}

impl Drop for GuardedJob {
    fn drop(&mut self) {
        if let Some(drop_callback) = self.callback.take() {
            drop_callback(GuardedJobStatus::Dropped, self.job.clone());
        }
    }
}

impl Deref for GuardedJob {
    type Target = vtask::AnonTask;

    fn deref(&self) -> &Self::Target {
        &self.job
    }
}

impl GuardedJob {
    /// *For use with Runtimes!*
    ///
    /// Create a new guarded job for this `AnonTask`
    pub fn new<F>(job: vtask::AnonTask, callback: Option<F>) -> Self
    where
        F: FnOnce(GuardedJobStatus, vtask::AnonTask) + Send + 'static,
    {
        let callback = callback.map(|callback| {
            let fn_ptr: Box<dyn FnOnce(GuardedJobStatus, vtask::AnonTask) + Send> =
                Box::new(callback);

            fn_ptr
        });

        Self { job, callback }
    }

    /// *For use with runners!*
    ///
    /// Mark this job as completed, calling the callback if one exists.
    pub fn mark_completed(mut self) {
        if let Some(callback) = self.callback.take() {
            callback(GuardedJobStatus::Finished, self.job.clone());
        }
    }

    /// *For use with runners!*
    ///
    /// Mark this job as completed, calling the callback if one exists.
    pub fn mark_canceled(mut self) {
        if let Some(callback) = self.callback.take() {
            callback(GuardedJobStatus::Canceled, self.job.clone());
        }
    }
}
