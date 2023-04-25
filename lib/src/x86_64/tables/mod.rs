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

use crate::x86_64::tables::idt::InterruptFrame;
use crate::remove_interrupt;
use crate::attach_interrupt;
use crate::{debug_print, debug_println};

use lazy_static::lazy_static;
use spin::Mutex;
use core::arch::asm;

pub mod idt;

lazy_static! {
    pub static ref INTERRUPT_DT: Mutex<idt::Idt> = {
        let mut idt = idt::Idt::new();

        debug_print!("Checking IDT ... ");

        // Our little test handler to make sure it gets called when we cause a Divide by Zero
        fn test_interrupt_dev0(_iframe: InterruptFrame, _interrupt: u8, _error: Option<u64>) {
            debug_println!("OK");
        }

        attach_interrupt!(idt, test_interrupt_dev0, 0);

        idt.submit_entries().expect("Unable to load IDT!").load();

        // test the handler
        unsafe {
            asm!("int 0x0");
        }

        remove_interrupt!(idt, 0);

        idt.submit_entries().expect("Unable to load IDT!").load();

        Mutex::new(idt)
    };
}