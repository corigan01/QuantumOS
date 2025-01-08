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

use arch::{
    attach_irq,
    idt64::{
        ExceptionKind, InterruptDescTable, InterruptFlags, InterruptInfo, fire_debug_int, interrupt,
    },
    interrupts::enable_interrupts,
    pic8259::{pic_eoi, pic_remap},
};
use lldebug::{logln, sync::Mutex};

static INTERRUPT_TABLE: Mutex<InterruptDescTable> = Mutex::new(InterruptDescTable::new());

#[interrupt(0..256)]
fn main_handler(args: InterruptInfo) {
    logln!("Handler == {:#016x?}", args);

    if args.flags.exception_kind() == ExceptionKind::Abort {
        panic!("Interrupt -- {:?}", args.flags);
    }

    match args.flags {
        InterruptFlags::GeneralProtectionFault => panic!("GPF"),
        // IRQ
        InterruptFlags::Irq(irq_num) if irq_num - PIC_IRQ_OFFSET <= 16 => {
            logln!("EOI -- {}", irq_num - PIC_IRQ_OFFSET);
            unsafe { pic_eoi(irq_num - PIC_IRQ_OFFSET) };
        }
        _ => (),
    }
}

pub fn attach_interrupts() {
    let mut idt = INTERRUPT_TABLE.lock();
    attach_irq!(idt, main_handler);
    unsafe { idt.submit_table().load() };

    logln!("Attached Interrupts!");

    logln!("Checking Interrupts...");
    fire_debug_int();
    logln!("Interrupts Working!");
}

const PIC_IRQ_OFFSET: u8 = 0x20;

pub fn enable_pic() {
    unsafe {
        pic_remap(PIC_IRQ_OFFSET, PIC_IRQ_OFFSET + 8);
        enable_interrupts();
    }
}
