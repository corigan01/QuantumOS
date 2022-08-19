/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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

use crate::{ serial_println, serial_print };
use crate::arch_x86_64::idt::InterruptFrame;
use crate::arch_x86_64::idt::ExtraHandlerInfo;

pub fn general_isr(i_frame: InterruptFrame, interrupt_id: u8, error_code: Option<u64>) {
    let extra_info = ExtraHandlerInfo::new(interrupt_id);

    if extra_info.reserved_interrupt && !extra_info.should_handler_diverge {
        serial_println!("Reserved Fault was called!");
        return;
    }

    serial_println!("\n\n=== FAULT HANDLER CALLED! ===");
    serial_println!("{} was called with an error code of {:#?}!",
        extra_info.interrupt_name, error_code);
    serial_println!("Interrupt Stack Frame:");
    serial_println!("\tCode Segment:        {}", i_frame.code_seg);
    serial_println!("\tInstruction Pointer: {:?}", i_frame.eip);
    serial_println!("\tFlags:               {}", i_frame.flags);
    serial_println!("\tStack Pointer:       {:?}", i_frame.stack_pointer);
    serial_println!("\tStack Segment:       {:?}", i_frame.stack_segment);


    if extra_info.should_handler_diverge {
        panic!("Diverging interrupt: {} was called!\n\t::{:#?}",
               extra_info.interrupt_name,
               error_code);
    }
}