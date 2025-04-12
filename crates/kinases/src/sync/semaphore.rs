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

use alloc::collections::VecDeque;
use alloc::sync::{Arc, Weak};

use crate::spin::mutex::SpinMutex;
use crate::spin::{DefaultSpin, SpinRelax};
use crate::wake::WakeCell;
use core::marker::PhantomData;
use core::mem;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, Waker};

pub struct Semaphore {
    state: SemaphoreState,
    // FIXME: This should be lock-less in the future.
    //
    // We could use a double ended linked-list where the tail is actually the 'start' of the queue, and
    // each time we enque an item, we CAS the 'head'-ptr with our new node.
    waker_queue: SpinMutex<VecDeque<Weak<SemaphoreRequestInner>>>,
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

            Ok(((available_tickets - n) << Self::CURRENT_TICKETS_OFFSET)
                | (current & !Self::CURRENT_TICKETS_MASK))
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

    fn return_tickets(&self, n: usize) -> usize {
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

            ((available_tickets + n) << Self::CURRENT_TICKETS_OFFSET)
                | (current & !Self::CURRENT_TICKETS_MASK)
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

        return ((new_current & Self::CURRENT_TICKETS_MASK) >> Self::CURRENT_TICKETS_OFFSET)
            as usize;
    }

    fn add_total_tickets(&self, n: usize) {
        let n = n as u64;
        let previous_value = self
            .0
            .fetch_add(n << Self::TOTAL_TICKETS_OFFSET, Ordering::SeqCst);

        assert!(
            Self::MAX_TICKETS as u64
                >= ((previous_value & Self::TOTAL_TICKETS_MASK) >> Self::TOTAL_TICKETS_OFFSET) + n,
            "Cannot add more than MAX_TICKETS! (Overflow)"
        );
    }

    fn sub_total_tickets(&self, n: usize) {
        let n = n as u64;
        let previous_value = self
            .0
            .fetch_sub(n << Self::TOTAL_TICKETS_OFFSET, Ordering::SeqCst);

        assert!(
            ((previous_value & Self::TOTAL_TICKETS_MASK) >> Self::TOTAL_TICKETS_OFFSET) >= n,
            "Cannot subtract more than the current max amount of tickets! (Underflow)"
        );
    }

    fn total_tickets(&self) -> usize {
        ((self.0.load(Ordering::Relaxed) & Self::TOTAL_TICKETS_MASK) >> Self::TOTAL_TICKETS_OFFSET)
            as usize
    }

    fn available_tickets(&self) -> usize {
        ((self.0.load(Ordering::Relaxed) & Self::CURRENT_TICKETS_MASK)
            >> Self::CURRENT_TICKETS_OFFSET) as usize
    }
}

impl Semaphore {
    pub const MAX_TICKETS: usize = SemaphoreState::MAX_TICKETS;

    pub const fn new(inital_tickets: usize) -> Self {
        assert!(inital_tickets <= Self::MAX_TICKETS);

        Self {
            state: SemaphoreState::new(inital_tickets),
            waker_queue: SpinMutex::new(VecDeque::new()),
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
        let mut waker_queue = self.waker_queue.lock();

        let Some(mut next_waker) = waker_queue.pop_front() else {
            // If our waker queue is empty, we can drop the lock now and return the tickets
            // without holding the lock.
            drop(waker_queue);

            self.state.return_tickets(quantity);
            return;
        };

        let current_tickets = self.state.return_tickets(quantity);

        loop {
            // Keep polling from the dequeue until we get a valid (non-dropped) waker
            let upgraded_waker = loop {
                if let Some(upgraded_waker) = next_waker.upgrade() {
                    break upgraded_waker;
                }

                let Some(new_next) = waker_queue.pop_front() else {
                    return;
                };

                next_waker = new_next;
            };

            // If we do not have enough tickets for this waker, we put it back in the queue
            let requested_tickets = upgraded_waker.requested_tickets();
            if requested_tickets > current_tickets {
                waker_queue.push_front(Arc::downgrade(&upgraded_waker));
                return;
            }

            // Otherwise if we have a waker that can accept the new tickets, we will aquire them now.
            //
            // This should always be valid, but if it fails we should just ignore it.
            if let Err(_) = self.state.try_take_tickets(requested_tickets) {
                waker_queue.push_front(Arc::downgrade(&upgraded_waker));
                return;
            }

            // At this point, we have requsted the tickets and reserved them. Now we need to call the waker
            // if one exists.
            if upgraded_waker.wake_remote() {
                // If the waker 'accepted' our wake request, then we can break out of the loop!
                break;
            }

            // If this waker couldn't be woken up, we will try another.
            //
            // We use `Weak::new()` to make sure the loop that checks for another waker fails to
            // upgrade, and thus will try to pop another from the queue.
            next_waker = Weak::new();
        }
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

    fn queue_task(
        &self,
        task: Arc<SemaphoreRequestInner>,
    ) -> Result<Option<AquiredTickets<'_>>, SemaphoreError> {
        let mut waker_queue = self.waker_queue.lock();
        let requested_tickets = task.requested_tickets();

        // If the task requested more tickets then the amount we currently have, we know its going
        // to be a failure.
        if self.quantity_total() < requested_tickets {
            return Err(SemaphoreError::NotEnoughTotalTickets);
        }

        // If there are wakers in the queue, we just need to be added to the end.
        if !waker_queue.is_empty() {
            waker_queue.push_back(Arc::downgrade(&task));

            // Return that we are not ready yet!
            return Ok(None);
        }

        // If there are no wakers in the queue, then lets attempt to make this request now.
        match self.state.try_take_tickets(requested_tickets) {
            Ok(_) => {
                task.set_dropping();

                Ok(Some(AquiredTickets {
                    amount: requested_tickets,
                    owner: self,
                }))
            }
            Err(SemaphoreError::WaitingOnEnoughTickets) => {
                waker_queue.push_back(Arc::downgrade(&task));

                Ok(None)
            }
            Err(err) => Err(err),
        }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        self.state.close();
    }
}

struct SemaphoreRequestInner {
    state: AtomicU64,
    waker: WakeCell,
}

impl SemaphoreRequestInner {
    const ATTACHED_FUTURE_BIT: u64 = 1 << (u64::BITS - 1);
    const READY_BIT: u64 = 1 << (u64::BITS - 2);
    const ACK_BIT: u64 = 1 << (u64::BITS - 3);
    const DROPPED_BIT: u64 = 1 << (u64::BITS - 4);

    const N_TICKETS_MASK: u64 = u64::MAX >> 4;

    const fn new(n_tickets: usize) -> Self {
        Self {
            state: AtomicU64::new((n_tickets as u64) & Self::N_TICKETS_MASK),
            waker: WakeCell::new(),
        }
    }

    fn attach_waker(&self, waker: Waker) {
        self.waker.attach(waker);
        let previous_state = self
            .state
            .fetch_or(Self::ATTACHED_FUTURE_BIT, Ordering::SeqCst);

        assert!(
            previous_state & Self::ATTACHED_FUTURE_BIT == 0,
            "Expected only one future to be attached!"
        );
    }

    fn requested_tickets(&self) -> usize {
        (self.state.load(Ordering::Relaxed) & Self::N_TICKETS_MASK) as usize
    }

    fn wake_remote(self: Arc<Self>) -> bool {
        let mut current = self.state.load(Ordering::Relaxed);

        // If the ready bit has already been set, we don't need to do anything.
        if current & Self::READY_BIT != 0 {
            return false;
        }

        // If this requst is currently being dropped, don't make the
        if current & Self::DROPPED_BIT != 0 {
            return false;
        }

        while let Err(failed) = self.state.compare_exchange_weak(
            current,
            current | Self::READY_BIT,
            Ordering::SeqCst,
            Ordering::Relaxed,
        ) {
            // If the ready bit has already been set, we don't need to do anything
            if failed & Self::READY_BIT != 0 {
                return false;
            }

            // If this requst is currently being dropped, don't make the
            if current & Self::DROPPED_BIT != 0 {
                return false;
            }

            current = failed;
        }

        // If we are the thread that set the READY_BIT then we can call the waker.
        self.waker.wake();

        true
    }

    fn is_ready(&self) -> Result<bool, SemaphoreError> {
        let mut current = self.state.load(Ordering::Relaxed);

        // If we have already ack this ready, then we don't do it again
        if current & Self::ACK_BIT != 0 {
            return Err(SemaphoreError::Closed);
        }

        if current & Self::DROPPED_BIT != 0 {
            return Err(SemaphoreError::Closed);
        }

        while let Err(failed) = self.state.compare_exchange_weak(
            current | Self::READY_BIT,
            current | Self::ACK_BIT,
            Ordering::SeqCst,
            Ordering::Relaxed,
        ) {
            // Check again to make sure no one else else has attempted to check the ready state
            // of our value.
            if failed & Self::ACK_BIT != 0 {
                return Err(SemaphoreError::Closed);
            }

            if current & Self::DROPPED_BIT != 0 {
                return Err(SemaphoreError::Closed);
            }

            // If the ready bit is not set, then we are not ready to read the value yet.
            if failed & Self::READY_BIT == 0 {
                return Ok(false);
            }

            current = failed;
        }

        // If we exit the CAS loop, then that means we are the ones that ACK'ed the ready state,
        // so we get to return true!
        return Ok(true);
    }

    fn set_dropping(&self) {
        self.state.fetch_or(Self::DROPPED_BIT, Ordering::SeqCst);
    }
}

#[must_use = "Aquire operations are lazy until polled or blocked."]
pub struct PendingSemaphoreAquire<'a, R: SpinRelax = DefaultSpin> {
    semaphore: &'a Semaphore,
    n_tickets: usize,
    request: Option<Arc<SemaphoreRequestInner>>,
    // We don't want this type to be sent between threads.
    ph: PhantomData<*mut R>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemaphoreError {
    NotEnoughTotalTickets,
    WaitingOnEnoughTickets,
    Poisoned,
    Closed,
    AlreadyWaiting,
}

impl<'a, R: SpinRelax> PendingSemaphoreAquire<'a, R> {
    fn new(semaphore: &'a Semaphore, n_tickets: usize) -> Self {
        Self {
            semaphore,
            request: None,
            n_tickets,
            ph: PhantomData,
        }
    }

    fn enqueue_in_semaphore(
        &mut self,
        waker: Option<Waker>,
    ) -> Result<Option<AquiredTickets<'a>>, SemaphoreError> {
        assert!(
            self.request.is_none(),
            "Making a request should only be called by the future impl."
        );

        let inner_requeset = SemaphoreRequestInner::new(self.n_tickets);
        if let Some(waker) = waker {
            inner_requeset.attach_waker(waker);
        }

        let waker_arc = Arc::new(inner_requeset);

        self.request = Some(waker_arc.clone());
        self.semaphore.queue_task(waker_arc)
    }

    pub fn try_aquire(&mut self) -> Result<Option<AquiredTickets<'a>>, SemaphoreError> {
        let Some(ref request) = self.request else {
            return self.enqueue_in_semaphore(None);
        };

        match request.is_ready() {
            Ok(true) => Ok(Some(AquiredTickets {
                amount: self.n_tickets,
                owner: self.semaphore,
            })),
            Ok(false) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn blocking_aquire(mut self) -> Result<AquiredTickets<'a>, SemaphoreError> {
        loop {
            match self.try_aquire() {
                Ok(Some(aquired)) => break Ok(aquired),
                Ok(None) => (),
                Err(err) => break Err(err),
            }

            R::back_off();
        }
    }
}

impl<'a> Future for PendingSemaphoreAquire<'a> {
    type Output = Result<AquiredTickets<'a>, SemaphoreError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Some(ref request) = self.request else {
            // If we have not made a request to the semaphore, we need to make one now.
            //
            // There are three possible outputs from making a request:
            //  1. The request is ready now.
            //  2. The request is valid but needs to be woken up.
            //  3. The request is invalid.
            match self.enqueue_in_semaphore(Some(cx.waker().clone())) {
                Ok(Some(aquired_tickets)) => return Poll::Ready(Ok(aquired_tickets)),
                Ok(None) => return Poll::Pending,
                Err(err) => return Poll::Ready(Err(err)),
            }
        };

        // If we have already made a request, then we just have to check if its
        // ready.
        match request.is_ready() {
            Ok(true) => Poll::Ready(Ok(AquiredTickets {
                amount: self.n_tickets,
                owner: self.semaphore,
            })),
            Ok(false) => Poll::Pending,
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl<'a, R: SpinRelax> Drop for PendingSemaphoreAquire<'a, R> {
    fn drop(&mut self) {
        // If the semaphore gave us tickets and we had not ack'ed them, we need to give them back now.
        if let Some(ref request) = self.request {
            match request.is_ready() {
                // If we are 'ready' that means we had not accepted our tickets, we lets do that now
                // and release them.
                Ok(true) => {
                    self.semaphore.release(self.n_tickets);
                }
                _ => (),
            }
        }
    }
}

/// Tickets borrowed from the Semaphore.
///
/// Once this type is dropped, the tickets will automaticlly be given back to the semaphore. No
/// manual clean-up is required for giving tickets back.
///
/// # Note
/// Semaphore's will use 'release' as a wake point for next `PendingSemaphoreAquire` requests, and
/// this is no exception. The drop duration of this type could be significant.
pub struct AquiredTickets<'a> {
    amount: usize,
    owner: &'a Semaphore,
}

impl<'a> Drop for AquiredTickets<'a> {
    fn drop(&mut self) {
        self.owner.release(self.amount);
    }
}

/// Tickets taken from a semaphore that do not get returned. These tickets are 'forgotten' from
/// their owner, and can be used however the caller desires.
///
/// To return these tickets back to the semaphore, the caller must _add_ them back in the same
/// way you would increase the total ticket amount:
/// ```rust
/// use kinases::sync::semaphore::Semaphore;
///
/// fn main() {
///    let s = Semaphore::new(10);
///
///    let borrowed_ticket = s.aquire(1).blocking_aquire().unwrap();
///    let owned_ticket = borrowed_ticket.to_owned();
///
///    // Owned tickets 'forget' the tickets from their parent semaphore
///    assert_eq!(s.quantity_total(), 9);
///
///    // Add them back
///    s.add_tickets(owned_ticket.0);
/// }
/// ```
pub struct OwnedTickets(pub usize);

impl<'a> AquiredTickets<'a> {
    /// Removes tickets from the owner's semaphore's total quanity of tickets.
    ///
    /// This is not a normal aquire/release, this consumes the tickets!
    pub fn to_owned(self) -> OwnedTickets {
        let amount = self.amount;
        let owner = self.owner;

        // Don't drop self because we don't want to release our tickets, we want to take them!
        mem::forget(self);

        owner.remove_tickets(amount);
        OwnedTickets(amount)
    }
}

#[cfg(test)]
mod test {
    use super::Semaphore;
    use crate::sync::semaphore::SemaphoreError;
    use std::{sync::Arc, thread, vec::Vec};

    extern crate std;

    #[test]
    fn semaphore_aquire() {
        let s = Semaphore::new(10);

        assert_eq!(s.quantity_total(), 10);
        assert_eq!(s.quantity_available(), 10);

        for q in 0..10 {
            // Aquires are lazy
            let aquire = s.aquire(q);

            assert_eq!(s.quantity_total(), 10);
            assert_eq!(s.quantity_available(), 10);

            {
                let borrowed_tickets = aquire.blocking_aquire().unwrap();

                assert_eq!(s.quantity_total(), 10);
                assert_eq!(s.quantity_available(), 10 - q);

                drop(borrowed_tickets);
            }

            assert_eq!(s.quantity_total(), 10);
            assert_eq!(s.quantity_available(), 10);
        }
    }

    #[test]
    fn semaphore_subtract_total() {
        let s = Semaphore::new(10);

        s.aquire(10).blocking_aquire().unwrap().to_owned();
    }

    #[test]
    fn test_multithreaded() {
        let s = Arc::new(Semaphore::new(10));

        #[cfg(not(miri))]
        const MAX_THREADS: usize = 32;
        #[cfg(not(miri))]
        const MAX_THREAD_ITER: usize = 1000;

        // This is used otherwise miri takes forever to run
        #[cfg(miri)]
        const MAX_THREADS: usize = 2;
        #[cfg(miri)]
        const MAX_THREAD_ITER: usize = 100;

        let mut thread_joins = Vec::new();
        for _ in 0..MAX_THREADS {
            let s = s.clone();
            thread_joins.push(thread::spawn(move || {
                // Make sure all the threads stay busy for a bit
                for _ in 0..MAX_THREAD_ITER {
                    let holding_tickets = s.aquire(1).blocking_aquire().unwrap();

                    assert_eq!(s.quantity_total(), 10);
                    assert!(s.quantity_available() <= 9);

                    drop(holding_tickets);
                }
            }));
        }

        for thread in thread_joins {
            thread.join().unwrap();
        }

        assert_eq!(s.quantity_total(), 10);
        assert_eq!(s.quantity_available(), 10);
    }

    #[test]
    fn test_fails() {
        let s = Semaphore::new(100);

        {
            let mut pending = s.aquire(200);
            assert!(matches!(
                pending.try_aquire(),
                Err(SemaphoreError::NotEnoughTotalTickets)
            ));
        }

        s.poison_semaphore();

        {
            let mut pending = s.aquire(20);
            assert!(matches!(
                pending.try_aquire(),
                Err(SemaphoreError::Poisoned)
            ));
        }

        unsafe { s.unpoison_semaphore() };

        {
            let mut pending = s.aquire(20);
            assert!(matches!(pending.try_aquire(), Ok(_)));
        }
    }

    #[test]
    fn test_ordering() {
        let s = Semaphore::new(10);

        let mut a = s.aquire(5);
        let mut b = s.aquire(10);
        let mut c = s.aquire(1);

        let aquired_a = a.try_aquire().unwrap().unwrap();

        assert!(matches!(b.try_aquire(), Ok(None)));

        // Ordering should make sure we wake `b` before `c` even though we could
        // populate `c` right now!
        assert!(matches!(c.try_aquire(), Ok(None)));

        drop(aquired_a);

        // Ordering should make sure we wake `b` before `c`
        assert!(matches!(c.try_aquire(), Ok(None)));

        let aquired_b = b.try_aquire().unwrap().unwrap();

        assert!(matches!(c.try_aquire(), Ok(None)));

        drop(aquired_b);

        let _ = c.try_aquire().unwrap().unwrap();
    }
}
