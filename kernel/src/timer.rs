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

use core::sync::atomic::{AtomicU64, Ordering};

use crate::{int::attach_irq_handler, process::scheduler::Scheduler};
use arch::{
    critcal_section,
    idt64::InterruptInfo,
    pit825x::{PitAccessMode, PitOperatingMode, PitSelectChannel, pit_command, set_pit_hz},
};
use lldebug::{log, logln};

const TIMER_HZ: f32 = 1000_f32;

pub fn init_timer() {
    log!("Enabling PIT...");
    critcal_section! {
        // Put the pit in repeted trigger mode
        pit_command(
            PitSelectChannel::Channel0,
            PitAccessMode::AccessLoHi,
            PitOperatingMode::SquareWave,
            false,
        );

        // Set the trigger time
        log!("({}Hz)", set_pit_hz(TIMER_HZ));

        // Attach our IRQ
        attach_irq_handler(pit_interrupt_handler, 0);
    }
    logln!("OK");
}

static KERNEL_TICKS: AtomicU64 = AtomicU64::new(0);

fn pit_interrupt_handler(_args: &InterruptInfo) {
    KERNEL_TICKS.fetch_add(1, Ordering::AcqRel);
    Scheduler::yield_me();
}

pub fn kernel_ticks() -> u64 {
    KERNEL_TICKS.load(Ordering::Relaxed)
}
