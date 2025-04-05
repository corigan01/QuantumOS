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

use alloc::{
    collections::{btree_set::BTreeSet, vec_deque::VecDeque},
    sync::Arc,
};
use core::sync::atomic::{AtomicBool, Ordering};
use kinases::spin::mutex::SpinMutex;
use runner::TaskRunner;
use runtime::{GuardedJob, GuardedJobStatus, RuntimeSupport};
use task::Task;

extern crate alloc;

pub mod runner;
pub mod runtime;
pub mod task;
pub mod vtask;

#[derive(Clone)]
pub struct Chloroplast {
    needs_poll: Arc<SpinMutex<VecDeque<vtask::AnonTask>>>,
    waiting: Arc<SpinMutex<BTreeSet<vtask::AnonTask>>>,
    shutting_down: Arc<AtomicBool>,
}

impl RuntimeSupport for Chloroplast {
    fn task_awoken(&self, task: vtask::AnonTask) {
        self.needs_poll.lock().push_back(task);
    }

    fn next_awaiting_task(&self) -> Option<GuardedJob> {
        { self.needs_poll.lock().pop_front() }.map(|job| {
            let tasks_clone = self.waiting.clone();
            GuardedJob::new(
                job,
                Some(move |reason, job| match reason {
                    GuardedJobStatus::Finished => {
                        tasks_clone.lock().remove(&job);
                    }
                    GuardedJobStatus::Dropped => {
                        tasks_clone.lock().insert(job);
                    }
                    GuardedJobStatus::Canceled => {
                        tasks_clone.lock().remove(&job);
                    }
                }),
            )
        })
    }

    fn runtime_status(&self) -> runtime::RuntimeStatus {
        if self.shutting_down.load(Ordering::Relaxed) {
            runtime::RuntimeStatus::ShuttingDown
        } else {
            runtime::RuntimeStatus::Running
        }
    }
}

impl Chloroplast {
    pub fn new() -> Self {
        Self {
            needs_poll: Arc::new(SpinMutex::new(VecDeque::new())),
            waiting: Arc::new(SpinMutex::new(BTreeSet::new())),
            shutting_down: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn spawn<F>(&self, future: F) -> Task<F, Self, F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let new_task = Task::new(future, self.clone());
        self.needs_poll.lock().push_back(new_task.anon_task());

        new_task
    }

    pub fn new_runner(&self) -> TaskRunner<Self> {
        TaskRunner::new(self.clone())
    }

    pub fn is_work_finished(&self) -> bool {
        self.needs_poll.lock().len() == 0 && self.shutting_down.load(Ordering::Relaxed)
    }

    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let new_task = Task::new(future, self.clone());
        self.needs_poll.lock().push_back(new_task.anon_task());

        let mut runner = self.new_runner();
        while !new_task.is_completed() {
            runner.drive_execution();
        }

        new_task
            .raw_task()
            .get_output()
            .expect("Expected task to return output!")
    }

    pub fn shutdown(&self) {
        self.shutting_down.store(true, Ordering::SeqCst);
    }
}

impl Drop for Chloroplast {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod test {
    use core::{
        pin::Pin,
        task::{Context, Poll},
    };
    extern crate std;
    use alloc::vec::Vec;

    use super::*;

    #[derive(Debug)]
    pub struct Yield(bool);

    impl Future for Yield {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if !self.0 {
                self.0 = true;
                cx.waker().wake_by_ref();

                Poll::Pending
            } else {
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

    #[test]
    fn test_runtime() {
        let yielding_future = Yield::new();

        let runtime = Chloroplast::new();
        runtime.spawn(async move {
            yielding_future.await;
            assert_eq!(test_async(10).await, 20);
        });

        runtime.spawn(async move {
            Yield::new().await;
            Yield::new().await;
            assert_eq!(test_async(0).await, 10);
        });

        let mut runner = runtime.new_runner();
        while !runtime.is_work_finished() {
            runner.drive_execution();
        }
    }

    #[test]
    fn test_runtime_blocking() {
        let runtime = Chloroplast::new();

        assert_eq!(
            runtime.block_on(async {
                for _ in 0..10 {
                    Yield::new().await;
                }

                test_async(10).await
            }),
            20
        );
    }

    #[test]
    fn test_multi_threading() {
        let runtime = Chloroplast::new();

        let mut threads = Vec::new();
        for _ in 0..4 {
            let runtime = runtime.clone();
            threads.push(std::thread::spawn(move || {
                let mut runner = runtime.new_runner();

                while !runtime.is_work_finished() {
                    runner.drive_execution();
                }
            }));
        }

        for _ in 0..100 {
            runtime.spawn(async move {
                for _ in 0..10 {
                    Yield::new().await;
                }
            });
        }

        runtime.shutdown();

        for thread in threads {
            thread.join().unwrap();
        }
    }
}
