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

use core::{
    pin::Pin,
    task::{Context, Poll},
};

use alloc::{collections::vec_deque::VecDeque, sync::Arc, vec::Vec};
use lldebug::logln;
use sync::spin::mutex::SpinMutex;
use task::Task;
use vtask::RuntimeSupport;

extern crate alloc;

pub mod runner;
pub mod task;
pub mod vtask;
pub mod wake;

#[derive(Debug)]
pub struct Yield(bool);

impl Future for Yield {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        logln!("Yield is being polled!");
        if !self.0 {
            self.0 = true;
            cx.waker().wake_by_ref();

            logln!("Yield -> Pending");
            Poll::Pending
        } else {
            logln!("Yield -> Ready");
            Poll::Ready(())
        }
    }
}

impl Yield {
    pub const fn new() -> Self {
        Self(false)
    }
}

async fn test_async(dingus: i32) -> i32 {
    dingus + 10
}

struct Runtime {
    tasks: SpinMutex<VecDeque<vtask::AnonTask>>,
}

impl RuntimeSupport for Arc<Runtime> {
    fn schedule_task(&self, task: vtask::AnonTask) {
        logln!("Wake called!");
        self.tasks.lock().push_back(task);
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            tasks: SpinMutex::new(VecDeque::new()),
        }
    }

    pub fn spawn<F>(self: &Arc<Self>, future: F) -> Task<F, Arc<Self>, F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let new_task = Task::spawn(future, self.clone());
        self.tasks.lock().push_back(new_task.anon_task());

        new_task
    }

    pub fn block_all(self: &Arc<Self>) {
        while let Some(next) = { self.tasks.lock().pop_front() } {
            unsafe { next.vtable_run() };
        }
    }
}

#[cfg(test)]
mod test {
    use lldebug::{logln, testing_stdout};

    use super::*;

    #[test]
    fn test_runtime() {
        testing_stdout!();
        let yielding_future = Yield::new();

        let runtime = Arc::new(Runtime::new());
        let task = runtime.clone().spawn(yielding_future);
        runtime.block_all();

        logln!("{:#?}", task.raw_task());

        logln!("\n ");
    }
}
