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

use crate::{
    runtime::{RuntimeStatus, RuntimeSupport},
    vtask::{self},
};

/// Single execution unit for the runtime.
///
/// A `TaskRunner` is a work-stealing single execution enviroment for tasks
/// to be ran within.
pub struct TaskRunner<Run> {
    runtime: Run,
}

impl<Run: RuntimeSupport> TaskRunner<Run> {
    pub fn new(runtime: Run) -> Self {
        Self { runtime }
    }

    pub fn drive_execution(&mut self) {
        let Some(next_job) = self.runtime.next_awaiting_task() else {
            return;
        };

        match unsafe { next_job.vtable_run() } {
            vtask::RunResult::Pending
                if self.runtime.runtime_status() == RuntimeStatus::ShuttingDown =>
            {
                // If the runtime is shutting down, we should mark that in the task
                unsafe { next_job.vtable_override_status(vtask::RunResult::Canceled) };
                next_job.mark_canceled();
            }
            vtask::RunResult::Pending => (),
            vtask::RunResult::Finished => next_job.mark_completed(),
            vtask::RunResult::Canceled => next_job.mark_canceled(),
        }
    }
}
