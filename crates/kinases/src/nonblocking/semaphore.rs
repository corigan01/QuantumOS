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

use crate::{spin::mutex::SpinMutex, wake::WakeQueue};
use alloc::collections::{BTreeMap, VecDeque};
use core::sync::atomic::AtomicUsize;
use core::task::Waker;

pub struct Semaphore {
    state: AtomicUsize,
    fair: SpinMutex<VecDeque<(usize, Option<Waker>)>>,
    unfair: SpinMutex<BTreeMap<usize, WakeQueue>>,
}

impl Semaphore {
    const POISONED_BIT: usize = 1 << usize::BITS - 1;
    const TICKETS_MASK: usize = usize::MAX >> 1;

    pub const fn new(inital_tickets: usize) -> Self {
        assert!(inital_tickets < Self::TICKETS_MASK);

        Self {
            state: AtomicUsize::new(inital_tickets),
            fair: SpinMutex::new(VecDeque::new()),
            unfair: SpinMutex::new(BTreeMap::new()),
        }
    }

    pub async fn aquire(&self, quantity: usize) -> AquiredTickets<'_> {
        todo!()
    }

    pub async fn unfairly_aquire(&self, quantity: usize) -> AquiredTickets<'_> {
        todo!()
    }

    pub fn try_aquire(&self, quantity: usize) -> Option<AquiredTickets<'_>> {
        todo!()
    }

    pub fn try_unfairly_aquire(&self, quantity: usize) -> Option<AquiredTickets<'_>> {
        todo!()
    }

    pub fn blocking_aquire(&self, quantity: usize) -> AquiredTickets<'_> {
        todo!()
    }

    pub fn blocking_unfairly_aquire(&self, quantity: usize) -> AquiredTickets<'_> {
        todo!()
    }

    pub fn release(&self, quantity: usize) {
        todo!()
    }

    pub fn forget_tickets(&self, quantity: usize) {
        todo!()
    }

    pub fn quantity_available(&self) -> usize {
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
