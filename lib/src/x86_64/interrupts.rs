/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
  Part of the Quantum OS Project

Copyright 2023 Gavin Kellam

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

use crate::x86_64::registers::EFLAGS;
use core::arch::asm;

pub struct Interrupts {}

impl Interrupts {
    /// # Safety
    /// The caller must ensure that disabling interrupts are valid, and will not cause issues.
    #[inline(always)]
    pub unsafe fn disable() {
        asm!("cli");
    }

    /// # Safety
    /// The caller must ensure that enabling interrupts are valid, and will not cause issues.
    #[inline(always)]
    pub unsafe fn enable() {
        asm!("sti");
    }

    pub fn assert_interrupts_must_be_enabled() {
        let enabled = EFLAGS::is_interrupt_enable_flag_set();

        assert!(
            enabled,
            "Expected Interrupts to be enabled, try `Interrupts::enable()`!"
        );
    }

    pub fn assert_interrupts_must_be_disabled() {
        let enabled = EFLAGS::is_interrupt_enable_flag_set();

        assert!(
            !enabled,
            "Expected Interrupts to be disabled, try `Interrupts::disable()`!"
        );
    }
}
