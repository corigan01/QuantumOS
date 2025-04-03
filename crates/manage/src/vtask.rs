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
    alloc::Layout,
    cell::UnsafeCell,
    fmt::Debug,
    mem::ManuallyDrop,
    pin::Pin,
    ptr::drop_in_place,
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

type OpaquePtr = *const ();

pub enum RunResult {
    Pending,
    Finished,
    Canceled,
}

pub struct FnTable {
    wake: unsafe fn(OpaquePtr),
    wake_ref: unsafe fn(OpaquePtr),
    clone_waker: unsafe fn(OpaquePtr) -> RawWaker,
    drop: unsafe fn(OpaquePtr),
    run: unsafe fn(OpaquePtr) -> RunResult,
    output: unsafe fn(OpaquePtr) -> OpaquePtr,
    set_waker: unsafe fn(OpaquePtr, Waker),
}

#[repr(C)]
struct TaskMem<Fut, Run, Out> {
    state: TaskState,
    vtable: FnTable,
    waker: UnsafeCell<Option<Waker>>,
    future: UnsafeCell<Fut>,
    output: UnsafeCell<Option<Out>>,
    runtime: UnsafeCell<Run>,
}

pub struct TaskState(AtomicUsize);

impl TaskState {
    const REF_COUNT_MAX: usize = usize::MAX >> 4;

    const FINISHED_BIT: usize = 1 << usize::BITS - 1;
    const RUNNING_BIT: usize = 1 << usize::BITS - 2;
    const CANCEL_BIT: usize = 1 << usize::BITS - 3;
    const CONSUMED_BIT: usize = 1 << usize::BITS - 4;

    pub const fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    pub fn add_ref(&self) -> usize {
        let old_count = self.0.fetch_add(1, Ordering::SeqCst);
        assert!(old_count < Self::REF_COUNT_MAX);

        old_count + 1
    }

    pub fn sub_ref(&self) -> usize {
        let old_count = self.0.fetch_sub(1, Ordering::SeqCst);
        assert!(old_count > 0);

        old_count - 1
    }

    pub fn try_consume(&self) -> bool {
        let mut current = self.0.load(Ordering::Relaxed);
        while let Err(failed) = self.0.compare_exchange_weak(
            current & Self::REF_COUNT_MAX | Self::FINISHED_BIT,
            current | Self::CONSUMED_BIT,
            Ordering::SeqCst,
            Ordering::Relaxed,
        ) {
            if failed & Self::FINISHED_BIT == 0 {
                // Not finished
                return false;
            }

            if failed & Self::CANCEL_BIT != 0 {
                // Future Canceled
                return false;
            }

            if failed & Self::CONSUMED_BIT != 0 {
                // Already consumed
                return false;
            }

            current = failed;
        }

        true
    }

    pub fn poll_lifecycle<F>(&self, poll_fun: F) -> RunResult
    where
        F: FnOnce() -> RunResult,
    {
        let mut current = self.0.load(Ordering::Relaxed);

        while let Err(failed) = self.0.compare_exchange_weak(
            (current & Self::REF_COUNT_MAX),
            current | Self::RUNNING_BIT,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            assert!(
                failed & Self::REF_COUNT_MAX > 0,
                "Tried to pull a dropped future!"
            );

            if failed & Self::FINISHED_BIT != 0 {
                // Future already finished
                return RunResult::Finished;
            }

            if failed & Self::CANCEL_BIT != 0 {
                // Future cancled
                return RunResult::Canceled;
            }

            if failed & Self::RUNNING_BIT != 0 {
                // Future already running
                return RunResult::Pending;
            }

            current = failed;
        }

        // Run the future
        match poll_fun() {
            RunResult::Finished => {
                self.0
                    .fetch_xor(Self::RUNNING_BIT | Self::FINISHED_BIT, Ordering::Release);

                RunResult::Finished
            }
            RunResult::Canceled => {
                self.0
                    .fetch_xor(Self::RUNNING_BIT | Self::CANCEL_BIT, Ordering::Release);

                RunResult::Canceled
            }
            RunResult::Pending => {
                self.0.fetch_xor(Self::RUNNING_BIT, Ordering::Release);

                RunResult::Pending
            }
        }
    }
}

impl<Fut, Run, Out> TaskMem<Fut, Run, Out> {
    #[inline]
    unsafe fn write_init(
        this: *mut Self,
        state: TaskState,
        vtable: FnTable,
        future: Fut,
        runtime: Run,
    ) {
        unsafe {
            (&raw mut (*this).state).write(state);
            (&raw mut (*this).vtable).write(vtable);
            (&raw mut (*this).waker).write(UnsafeCell::new(None));
            (&raw mut (*this).future).write(UnsafeCell::new(future));
            (&raw mut (*this).runtime).write(UnsafeCell::new(runtime));
            (&raw mut (*this).output).write(UnsafeCell::new(None));
        }
    }

    const fn layout() -> Layout {
        Layout::new::<Self>()
    }

    #[inline]
    unsafe fn allocate_new() -> *mut Self {
        let layout = Self::layout();
        let mem_ptr = unsafe { alloc::alloc::alloc_zeroed(layout) } as *mut Self;

        mem_ptr
    }

    #[inline]
    unsafe fn deallocate(this: *mut Self) {
        let layout = Self::layout();
        unsafe { alloc::alloc::dealloc(this.cast(), layout) };
    }
}

#[repr(transparent)]
pub struct AnonTask {
    mem_ptr: *mut TaskMem<(), (), ()>,
}

impl AnonTask {
    pub unsafe fn vtable_wake(&self) {
        unsafe { ((&*self.mem_ptr).vtable.wake)(self.mem_ptr.cast()) }
    }
    pub unsafe fn vtable_wake_ref(&self) {
        unsafe { ((&*self.mem_ptr).vtable.wake_ref)(self.mem_ptr.cast()) }
    }

    pub unsafe fn vtable_clone_waker(&self) -> RawWaker {
        unsafe { ((&*self.mem_ptr).vtable.clone_waker)(self.mem_ptr.cast()) }
    }

    pub unsafe fn vtable_drop(&self) {
        unsafe { ((&*self.mem_ptr).vtable.drop)(self.mem_ptr.cast()) }
    }

    pub unsafe fn vtable_run(&self) -> RunResult {
        unsafe { ((&*self.mem_ptr).vtable.run)(self.mem_ptr.cast()) }
    }

    pub unsafe fn vtable_output<Output>(&self) -> *const UnsafeCell<Option<Output>> {
        unsafe { ((&*self.mem_ptr).vtable.output)(self.mem_ptr.cast()).cast() }
    }
}

impl Drop for AnonTask {
    fn drop(&mut self) {
        unsafe { ((&*self.mem_ptr).vtable.drop)(self.mem_ptr.cast()) }
    }
}

pub trait RuntimeSupport {
    fn schedule_task(&self, task: AnonTask);
}

#[repr(transparent)]
pub struct RawTask<Fut, Run, Out> {
    mem_ptr: *mut TaskMem<Fut, Run, Out>,
}

impl<Fut, Run> RawTask<Fut, Run, Fut::Output>
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
    Run: RuntimeSupport + Send + Sync + 'static,
{
    const RUST_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone_waker_raw,
        Self::wake_raw,
        Self::wake_ref_raw,
        Self::drop_raw,
    );

    pub fn new_allocated(future: Fut, runtime: Run) -> Self {
        unsafe {
            let mem_ptr = TaskMem::<Fut, Run, Fut::Output>::allocate_new();

            TaskMem::write_init(
                mem_ptr,
                TaskState::new(),
                FnTable {
                    wake: Self::wake_raw,
                    wake_ref: Self::wake_ref_raw,
                    clone_waker: Self::clone_waker_raw,
                    drop: Self::drop_raw,
                    run: Self::run_raw,
                    output: Self::raw_output,
                    set_waker: Self::raw_set_waker,
                },
                future,
                runtime,
            );

            Self { mem_ptr }
        }
    }

    /// Convert this `OpaquePtr` into Self
    ///
    /// This function turns the `OpaquePtr` we get from vtable calls into `Self`.
    ///
    /// # Safety
    /// This type of `Self` needs to match the `OpaquePtr` exactly. Memory layout calculations are done
    /// based on generics, and if the wrong types are used this can lead to undefined behavior.
    ///
    /// This function attempts to check this by checking if the `ptr` is aligned, however, multiple types
    /// may still have the same layout and still cause UB.
    unsafe fn upgrade_opaque(ptr: OpaquePtr) -> ManuallyDrop<Self> {
        let mem_ptr: *mut TaskMem<Fut, Run, Fut::Output> = ptr.cast_mut().cast();
        assert!(!mem_ptr.is_null() && mem_ptr.is_aligned());

        ManuallyDrop::new(Self { mem_ptr })
    }

    /// Convert this `RawTask` into an `AnonTask`
    ///
    /// This task contains generic info about the exact future that is contained, however, the runtime
    /// often does not need this info. So, this function serves to convert this type into one that can
    /// just call the `vtable`.
    pub fn downgrade(self) -> AnonTask {
        AnonTask {
            mem_ptr: self.mem_ptr.cast(),
        }
    }

    /// Wake this future
    ///
    /// Inform the scheduler about this future being ready for another `poll()`.
    fn wake(&self) {
        let runtime_ref = unsafe { &*(&*self.mem_ptr).runtime.get() };
        let downgrade_self = self.clone().downgrade();
        Run::schedule_task(runtime_ref, downgrade_self)
    }

    /// Poll the future
    ///
    /// Pull the future if its `Pending` or simply return `Finished` if this future is done.
    ///
    /// # Note
    /// This function will never pull a future after success, and will always send `Finished` if
    /// poll is called multiple times after the completion of this future.
    fn poll(&self) -> RunResult {
        unsafe {
            let poll_lifecycle = (&*self.mem_ptr).state.poll_lifecycle(|| {
                let future = Pin::new_unchecked(&mut *(&*self.mem_ptr).future.get());
                let waker = self.waker();

                let mut context = Context::from_waker(&waker);
                match future.poll(&mut context) {
                    Poll::Ready(output) => {
                        (&*self.mem_ptr).output.get().write(Some(output));

                        RunResult::Finished
                    }
                    Poll::Pending => RunResult::Pending,
                }
            });

            // Notify our waker if one exists
            match poll_lifecycle {
                RunResult::Finished | RunResult::Canceled => self.call_waker(),
                _ => (),
            }

            poll_lifecycle
        }
    }

    /// Get a cloned waker instance from this task
    ///
    /// Increases the ref count (clone)'s the inner value.
    fn waker(&self) -> Waker {
        unsafe { Waker::from_raw(Self::clone_waker_raw(self as *const _ as *const ())) }
    }

    /// Calls the waker set for this task if one exists
    ///
    /// If another async task is waiting for this one to finish, we need to wake it up whenever we finish.
    fn call_waker(&self) {
        unsafe {
            if let Some(waker) = (&*self.mem_ptr).waker.get().replace(None) {
                waker.wake();
            }
        }
    }

    /// Set a waker for this task
    ///
    /// This waker will be called whenever this task is finished, or is canceled.
    pub fn set_waker(&self, waker: Waker) {
        unsafe {
            (&*self.mem_ptr).waker.get().write(Some(waker));
        }
    }

    /// Gets the output if it exists
    ///
    /// This will consume the output value from the future. This future once consumed
    /// cannot return its output again.
    pub fn get_output(self) -> Option<Fut::Output> {
        unsafe {
            if (&*self.mem_ptr).state.try_consume() {
                let inner_ptr = (&*self.mem_ptr).output.get();

                // Replace the value with `None`
                let read_value = inner_ptr.read();
                inner_ptr.write(None);

                read_value
            } else {
                None
            }
        }
    }

    unsafe fn wake_raw(ptr: OpaquePtr) {
        unsafe {
            let mut s = Self::upgrade_opaque(ptr);
            s.wake();

            ManuallyDrop::drop(&mut s);
        }
    }

    unsafe fn wake_ref_raw(ptr: OpaquePtr) {
        unsafe {
            let s = Self::upgrade_opaque(ptr);
            s.wake();
        }
    }

    unsafe fn clone_waker_raw(ptr: OpaquePtr) -> RawWaker {
        unsafe {
            let s = Self::upgrade_opaque(ptr);
            (&*s.mem_ptr).state.add_ref();

            RawWaker::new(ptr, &Self::RUST_WAKER_VTABLE)
        }
    }

    unsafe fn drop_raw(ptr: OpaquePtr) {
        unsafe {
            let s = Self::upgrade_opaque(ptr);
            let ref_count = (&*s.mem_ptr).state.sub_ref();

            if ref_count == 0 {
                drop_in_place((&*s.mem_ptr).future.get());
                drop_in_place((&*s.mem_ptr).output.get());
                drop_in_place((&*s.mem_ptr).runtime.get());

                TaskMem::deallocate(s.mem_ptr);
            }
        }
    }

    unsafe fn run_raw(ptr: OpaquePtr) -> RunResult {
        unsafe {
            let s = Self::upgrade_opaque(ptr);
            s.poll()
        }
    }

    unsafe fn raw_output(ptr: OpaquePtr) -> OpaquePtr {
        unsafe {
            let s = Self::upgrade_opaque(ptr);
            let output_ptr = &raw const ((*s.mem_ptr).output);

            output_ptr.cast()
        }
    }

    unsafe fn raw_set_waker(ptr: OpaquePtr, waker: Waker) {
        unsafe {
            let s = Self::upgrade_opaque(ptr);
            s.set_waker(waker);
        }
    }
}

impl<Fut, Run, Out> Clone for RawTask<Fut, Run, Out> {
    fn clone(&self) -> Self {
        unsafe { (&*self.mem_ptr).state.add_ref() };

        Self {
            mem_ptr: self.mem_ptr,
        }
    }
}

impl<Fut, Run, Out> Drop for RawTask<Fut, Run, Out> {
    fn drop(&mut self) {
        unsafe { ((&*self.mem_ptr).vtable.drop)(self.mem_ptr.cast()) };
    }
}

impl Debug for TaskState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let inner_value = self.0.load(Ordering::Relaxed);

        f.debug_struct("TaskState")
            .field("running", &(inner_value & Self::RUNNING_BIT != 0))
            .field("canceled", &(inner_value & Self::CANCEL_BIT != 0))
            .field("finished", &(inner_value & Self::FINISHED_BIT != 0))
            .field("ref_count", &(inner_value & Self::REF_COUNT_MAX))
            .finish()
    }
}

impl Debug for FnTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FnTable").finish_non_exhaustive()
    }
}

impl<Fut, Run, Out> Debug for RawTask<Fut, Run, Out> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe {
            f.debug_struct("RawTask")
                .field("state", &(&*self.mem_ptr).state)
                .field("vtable", &(&*self.mem_ptr).vtable)
                .field("future", &"...")
                .field(
                    "output",
                    if (&*((&*self.mem_ptr).output.get())).is_some() {
                        &"Some(...)"
                    } else {
                        &"None"
                    },
                )
                .field("future", &"...")
                .field("runtime", &"...")
                .finish_non_exhaustive()
        }
    }
}
