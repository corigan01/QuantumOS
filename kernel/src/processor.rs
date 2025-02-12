/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

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

use core::sync::atomic::{AtomicUsize, Ordering};

static CURRENT_THREAD_ID: AtomicUsize = AtomicUsize::new(0);
static CURRENT_PROCESS_ID: AtomicUsize = AtomicUsize::new(0);
static HANDLING_IRQ: AtomicUsize = AtomicUsize::new(0);
static HANDLING_CRITICAL: AtomicUsize = AtomicUsize::new(0);

/// Set the processor's current thread ID
pub fn set_current_thread_id(thread_id: usize) {
    CURRENT_THREAD_ID.store(thread_id, Ordering::Relaxed);
}

/// Get the processor's current thread ID
pub fn get_current_thread_id() -> usize {
    CURRENT_THREAD_ID.load(Ordering::Relaxed)
}

/// Set the processor's current process ID
pub fn set_current_process_id(process_id: usize) {
    CURRENT_PROCESS_ID.store(process_id, Ordering::Relaxed);
}

/// Get the processor's current process ID
pub fn get_current_process_id() -> usize {
    CURRENT_PROCESS_ID.load(Ordering::Relaxed)
}

/// Inform that we are begining an IRQ
pub fn notify_begin_irq() {
    HANDLING_IRQ.fetch_add(1, Ordering::Acquire);
}

/// Inform that we are leaving an IRQ
pub fn notify_end_irq() {
    HANDLING_IRQ.fetch_sub(1, Ordering::Release);
}

/// Get if the processor is currently in an IRQ
pub fn is_within_irq() -> bool {
    HANDLING_IRQ.load(Ordering::Relaxed) > 0
}

/// Inform that we are begining a critical section
pub fn notify_begin_critical() {
    HANDLING_CRITICAL.fetch_add(1, Ordering::Acquire);
}

/// Inform that we are leaving a critical section
pub fn notify_end_critical() {
    HANDLING_CRITICAL.fetch_sub(1, Ordering::Release);
}

/// Get if the processor is currently in a critical section
pub fn is_within_critical() -> bool {
    HANDLING_CRITICAL.load(Ordering::Relaxed) > 0
}

/// Assert that we are (in/out) of a critical section
pub fn assert_critical(expected: bool) {
    assert_eq!(
        is_within_critical(),
        expected,
        "Expected to be {} a critical section!",
        if expected { "in" } else { "out of" }
    );
}
