/*
   ___   __        _   __
  / _ | / /__  ___| | / /__ _______ _
 / __ |/ / _ \/ -_) |/ / -_) __/ _ `/
/_/ |_/_/\___/\__/|___/\__/_/  \_,_/

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

extern crate alloc;

use crate::spin::mutex::SpinMutex;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, Waker};

pub struct Semaphore {
    state: SemaphoreState,
    // FIXME: This should be lock-less in the future.
    //
    // We could use a double ended linked-list where the tail is actually the 'start' of the queue, and
    // each time we enque an item, we CAS the 'head'-ptr with our new node.
    waker_queue: SpinMutex<Option<NonNull<PendingSemaphoreAquire<'static>>>>,
}

struct SemaphoreState(AtomicU64);

impl SemaphoreState {
    const POISONED_BIT: u64 = 1 << u64::BITS - 1;
    const CLOSED_BIT: u64 = 1 << u64::BITS - 2;

    const CURRENT_TICKETS_OFFSET: usize = 0;
    const CURRENT_TICKETS_MASK: u64 =
        (u64::MAX >> ((u64::BITS / 2) + 1)) << Self::CURRENT_TICKETS_OFFSET;
    const TOTAL_TICKETS_OFFSET: usize = 32;
    const TOTAL_TICKETS_MASK: u64 = Self::CURRENT_TICKETS_MASK << Self::TOTAL_TICKETS_OFFSET;

    const MAX_TICKETS: usize = Self::CURRENT_TICKETS_MASK as usize;

    const fn new(inital_tickets: usize) -> Self {
        Self(AtomicU64::new(
            inital_tickets as u64 | ((inital_tickets as u64) << Self::TOTAL_TICKETS_OFFSET),
        ))
    }

    fn close(&self) {
        self.0.fetch_or(Self::CLOSED_BIT, Ordering::SeqCst);
    }

    fn poison(&self) {
        self.0.fetch_or(Self::POISONED_BIT, Ordering::SeqCst);
    }

    unsafe fn unpoison(&self) {
        self.0.fetch_and(!Self::POISONED_BIT, Ordering::SeqCst);
    }

    fn try_take_tickets(&self, n: usize) -> Result<(), SemaphoreError> {
        let n = n as u64;
        let cal_new_current = |current: u64| {
            if current & Self::CLOSED_BIT != 0 {
                return Err(SemaphoreError::Closed);
            }

            if current & Self::POISONED_BIT != 0 {
                return Err(SemaphoreError::Poisoned);
            }

            let total_tickets = (current & Self::TOTAL_TICKETS_MASK) >> Self::TOTAL_TICKETS_OFFSET;

            if total_tickets < n {
                return Err(SemaphoreError::NotEnoughTotalTickets);
            }

            let available_tickets =
                (current & Self::CURRENT_TICKETS_MASK) >> Self::CURRENT_TICKETS_OFFSET;

            if available_tickets < n {
                return Err(SemaphoreError::WaitingOnEnoughTickets);
            }

            Ok(available_tickets - n)
        };

        let mut current = self.0.load(Ordering::Relaxed);
        let mut new_current = cal_new_current(current)?;

        while let Err(failed) =
            self.0
                .compare_exchange_weak(current, new_current, Ordering::SeqCst, Ordering::Relaxed)
        {
            current = failed;
            new_current = cal_new_current(current)?;
        }

        Ok(())
    }

    fn return_tickets(&self, n: usize) {
        let n = n as u64;
        let cal_new_current = |current: u64| {
            let total_tickets = (current & Self::TOTAL_TICKETS_MASK) >> Self::TOTAL_TICKETS_OFFSET;
            let available_tickets =
                (current & Self::CURRENT_TICKETS_MASK) >> Self::CURRENT_TICKETS_OFFSET;

            assert!(
                total_tickets >= available_tickets + n,
                "Cannot return more tickets then the total! (Overflow)"
            );
            assert!(
                Self::MAX_TICKETS as u64 >= available_tickets + n,
                "Cannot have more tickets then MAX_TICKETS! (Overflow)"
            );

            available_tickets + n
        };

        let mut current = self.0.load(Ordering::Relaxed);
        let mut new_current = cal_new_current(current);

        while let Err(failed) =
            self.0
                .compare_exchange_weak(current, new_current, Ordering::SeqCst, Ordering::Relaxed)
        {
            current = failed;
            new_current = cal_new_current(current);
        }
    }

    fn add_total_tickets(&self, n: usize) {
        let n = n as u64;
        let previous_value = self
            .0
            .fetch_add(n << Self::TOTAL_TICKETS_OFFSET, Ordering::SeqCst);

        assert!(
            Self::MAX_TICKETS as u64 >= (previous_value & Self::TOTAL_TICKETS_MASK) + n,
            "Cannot add more than MAX_TICKETS! (Overflow)"
        );
    }

    fn sub_total_tickets(&self, n: usize) {
        let n = n as u64;
        let previous_value = self
            .0
            .fetch_sub(n << Self::TOTAL_TICKETS_OFFSET, Ordering::SeqCst);

        assert!(
            (previous_value & Self::TOTAL_TICKETS_MASK) <= n,
            "Cannot subtract more than the current max amount of tickets! (Underflow)"
        );
    }

    fn total_tickets(&self) -> usize {
        (self.0.load(Ordering::Relaxed) & Self::TOTAL_TICKETS_MASK >> Self::TOTAL_TICKETS_OFFSET)
            as usize
    }

    fn available_tickets(&self) -> usize {
        (self.0.load(Ordering::Relaxed)
            & Self::CURRENT_TICKETS_MASK >> Self::CURRENT_TICKETS_OFFSET) as usize
    }
}

impl Semaphore {
    pub const MAX_TICKETS: usize = SemaphoreState::MAX_TICKETS;

    pub const fn new(inital_tickets: usize) -> Self {
        assert!(inital_tickets <= Self::MAX_TICKETS);

        Self {
            state: SemaphoreState::new(inital_tickets),
            waker_queue: SpinMutex::new(None),
        }
    }

    pub fn poison_semaphore(&self) {
        self.state.poison();
    }

    pub unsafe fn unpoison_semaphore(&self) {
        unsafe { self.state.unpoison() };
    }

    pub fn aquire(&self, quantity: usize) -> PendingSemaphoreAquire<'_> {
        PendingSemaphoreAquire::new(self, quantity)
    }

    pub fn release(&self, quantity: usize) {
        self.state.return_tickets(quantity);
    }

    /// Remove tickets from the maxium possible tickets
    pub fn remove_tickets(&self, quantity: usize) {
        self.state.sub_total_tickets(quantity);
    }

    pub fn add_tickets(&self, quantity: usize) {
        self.state.add_total_tickets(quantity);
    }

    pub fn quantity_available(&self) -> usize {
        self.state.available_tickets()
    }

    pub fn quantity_total(&self) -> usize {
        self.state.total_tickets()
    }

    pub fn queue_pending<'a>(&self, pending: &mut PendingSemaphoreAquire<'a>) {
        todo!()
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        self.state.close();
    }
}

pub struct PendingSemaphoreAquire<'a> {
    semaphore: &'a Semaphore,
    state: AtomicU64,
    waker: UnsafeCell<MaybeUninit<Waker>>,

    next: Option<NonNull<Self>>,
    prev: Option<NonNull<Self>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemaphoreError {
    NotEnoughTotalTickets,
    WaitingOnEnoughTickets,
    Poisoned,
    Closed,
    AlreadyWaiting,
}

impl<'a> PendingSemaphoreAquire<'a> {
    const DESIRED_TICKETS_MASK: usize = SemaphoreState::MAX_TICKETS;

    const WAITING_BIT: usize = 1 << u64::BITS - 1;
    const SOME_WAKER_BIT: usize = 1 << u64::BITS - 2;
    const IN_LIST_BIT: usize = 1 << u64::BITS - 3;
    const WAKEUP_BIT: usize = 1 << u64::BITS - 4;

    fn new(semaphore: &'a Semaphore, desired_tickets: usize) -> Self {
        Self {
            semaphore,
            state: AtomicU64::new(desired_tickets as u64),
            waker: UnsafeCell::new(MaybeUninit::uninit()),
            next: None,
            prev: None,
        }
    }

    /// Called remotely to wakeup self
    fn from_semaphore_wakeup(&self) {
        let mut current = self.state.load(Ordering::Relaxed) & (!Self::WAKEUP_BIT as u64);
        let mut new = (current & !(Self::WAITING_BIT as u64)) | Self::WAKEUP_BIT as u64;

        while let Err(failed) =
            self.state
                .compare_exchange_weak(current, new, Ordering::SeqCst, Ordering::Relaxed)
        {
            // If we have already woken up, we don't need to do it again
            if failed & Self::WAKEUP_BIT as u64 != 0 {
                return;
            }

            current = failed;
            new = (current & !(Self::WAITING_BIT as u64 | Self::SOME_WAKER_BIT as u64))
                | Self::WAKEUP_BIT as u64;
        }

        // At this point we should have the wake-up bit set, so other wake-up calls should
        // never come through.
        //
        // Here we need to call the waker if one exists. No data races should be possible
        // since we are the only ones allowed to call the waker.
        if current & Self::SOME_WAKER_BIT as u64 == 0 {
            // Our waker is None
            return;
        }

        let read_waker = unsafe { self.waker.get().read() };

        // Remove the waker bit
        let previous_state = self
            .state
            .fetch_and(!Self::SOME_WAKER_BIT as u64, Ordering::SeqCst);

        assert!(
            previous_state & Self::SOME_WAKER_BIT as u64 != 0,
            "No other thread should try to write the waker bit during this time!"
        );

        // Call the waker, and drop our ref
        unsafe { read_waker.assume_init_read().wake() };
    }

    pub fn blocking_aquire(self) -> AquiredTickets<'a> {
        todo!()
    }

    pub fn try_aquire(
        &mut self,
        waker: Option<Waker>,
    ) -> Result<AquiredTickets<'a>, SemaphoreError> {
        let mut current = self.state.load(Ordering::Relaxed)
            & !(Self::WAITING_BIT as u64 | Self::WAKEUP_BIT as u64);
        let mut new = current | Self::WAITING_BIT as u64;

        while let Err(failed) =
            self.state
                .compare_exchange_weak(current, new, Ordering::SeqCst, Ordering::Relaxed)
        {
            // If we are already waiting on this ticket, we shouldn't begin waiting again.
            if failed & Self::WAITING_BIT as u64 != 0 {
                return Err(SemaphoreError::AlreadyWaiting);
            }

            // We have the wake up bit set, and can return the result
            if failed & Self::WAKEUP_BIT as u64 != 0 {
                return Ok(AquiredTickets {
                    amount: failed as usize & Self::DESIRED_TICKETS_MASK,
                    owner: self.semaphore,
                });
            }

            current = failed;
            new = current | Self::WAITING_BIT as u64;
        }

        // Here we should've set the WAITING_BIT, and now we should have exclusive access
        // to write to the structure.

        // Set the waker if we have one
        if let Some(waker) = waker {
            unsafe { self.waker.get().write(MaybeUninit::new(waker)) };
            self.state
                .fetch_or(Self::SOME_WAKER_BIT as u64, Ordering::SeqCst);
        }

        self.semaphore.queue_pending(self);

        todo!()
    }
}

impl<'a> Future for PendingSemaphoreAquire<'a> {
    type Output = Result<AquiredTickets<'a>, SemaphoreError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut current = self.state.load(Ordering::Relaxed);

        todo!()
    }
}

pub struct AquiredTickets<'a> {
    amount: usize,
    owner: &'a Semaphore,
}

pub struct OwnedTickets(usize);

impl<'a> AquiredTickets<'a> {
    fn to_owned(self) -> OwnedTickets {
        todo!()
    }
}
