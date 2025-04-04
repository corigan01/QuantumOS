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

use core::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    runtime::RuntimeSupport,
    vtask::{self, RawTask, RunResult},
};

#[derive(Debug)]
pub struct Task<Fut, Run, Out> {
    raw: RawTask<Fut, Run, Out>,
}

impl<Fut, Run> Task<Fut, Run, Fut::Output>
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
    Run: RuntimeSupport + Send + Sync + 'static,
{
    pub fn new(future: Fut, runtime: Run) -> Self {
        Self {
            raw: RawTask::new_allocated(future, runtime),
        }
    }

    pub fn is_completed(&self) -> bool {
        self.raw.is_completed()
    }

    pub fn anon_task(&self) -> vtask::AnonTask {
        self.raw.clone().downgrade()
    }

    pub fn raw_task(&self) -> vtask::RawTask<Fut, Run, Fut::Output> {
        self.raw.clone()
    }
}

impl<Fut, Run> Future for Task<Fut, Run, Fut::Output>
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
    Run: RuntimeSupport + Send + Sync + 'static,
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.raw.set_waker(cx.waker().clone());
        match unsafe { self.raw.clone().downgrade().vtable_run() } {
            RunResult::Pending => Poll::Pending,
            RunResult::Finished => Poll::Ready(
                self.raw
                    .clone()
                    .get_output()
                    .expect("Task polled `Ready` yet no output, was future double-ready polled?"),
            ),
            RunResult::Canceled => panic!("Task canceled!"),
        }
    }
}
