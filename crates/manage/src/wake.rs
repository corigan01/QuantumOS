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
    mem::ManuallyDrop,
    ops::Deref,
    ptr::null,
    task::{RawWaker, RawWakerVTable, Waker},
};

pub trait WakeSupport: Send + Clone + 'static {
    fn wake(self);
    fn wake_by_ref(&self);

    unsafe fn into_opaque(self) -> *const ();
    unsafe fn from_opaque(ptr: *const ()) -> Self;

    fn get_raw_waker(self) -> RawWaker {
        RawWaker::new(unsafe { Self::into_opaque(self) }, get_vtable::<Self>())
    }

    fn get_waker(self) -> Waker {
        unsafe { Waker::from_raw(self.get_raw_waker()) }
    }
}

pub fn get_vtable<W: WakeSupport>() -> &'static RawWakerVTable {
    &RawWakerVTable::new(
        raw_clone::<W>,
        raw_wake::<W>,
        raw_wake_by_ref::<W>,
        raw_drop::<W>,
    )
}

fn raw_clone<W: WakeSupport>(ptr: *const ()) -> RawWaker {
    assert_ne!(ptr, null());

    let waker = ManuallyDrop::new(unsafe { W::from_opaque(ptr) });
    let cloned = waker.deref().clone();
    let new_ptr = unsafe { cloned.into_opaque() };

    RawWaker::new(new_ptr, get_vtable::<W>())
}

fn raw_wake<W: WakeSupport>(ptr: *const ()) {
    assert_ne!(ptr, null());

    let waker = unsafe { W::from_opaque(ptr) };
    waker.wake();
}

fn raw_wake_by_ref<W: WakeSupport>(ptr: *const ()) {
    assert_ne!(ptr, null());

    let waker = ManuallyDrop::new(unsafe { W::from_opaque(ptr) });
    waker.wake_by_ref();
}

fn raw_drop<W: WakeSupport>(ptr: *const ()) {
    assert_ne!(ptr, null());

    _ = unsafe { W::from_opaque(ptr) };
}
