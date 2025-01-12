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
    attach_irq, critcal_section,
    idt64::{
        ExceptionKind, InterruptDescTable, InterruptFlags, InterruptInfo, fire_debug_int, interrupt,
    },
    interrupts::enable_interrupts,
    pic8259::{pic_eoi, pic_remap},
};
use lldebug::{logln, sync::Mutex};

static INTERRUPT_TABLE: Mutex<InterruptDescTable> = Mutex::new(InterruptDescTable::new());
static IRQ_HANDLERS: Mutex<[Option<fn(&InterruptInfo)>; 32]> = Mutex::new([None; 32]);

#[interrupt(0..256)]
fn main_handler(args: InterruptInfo) {
    match args.flags {
        // IRQ
        InterruptFlags::Irq(irq_num) if irq_num - PIC_IRQ_OFFSET <= 16 => {
            call_attached_irq(irq_num - PIC_IRQ_OFFSET, &args);
            unsafe { pic_eoi(irq_num - PIC_IRQ_OFFSET) };
        }
        InterruptFlags::Debug => (),
        exception => {
            panic!("UNHANDLED FAULT\n{:#016x?}", args)
        }
        _ => (),
    }

    if args.flags.exception_kind() == ExceptionKind::Abort {
        panic!("Interrupt -- {:?}", args.flags);
    }
}

fn call_attached_irq(irq_id: u8, args: &InterruptInfo) {
    let irq_handler = IRQ_HANDLERS.lock();

    if let Some(handler) = irq_handler
        .get((irq_id) as usize)
        .and_then(|&handler| handler)
    {
        handler(args);
    }
}

/// Set a function to be called whenever an irq is triggered.
pub fn attach_irq_handler(handler_fn: fn(&InterruptInfo), irq: u8) {
    critcal_section! {
        let mut irq_handler = IRQ_HANDLERS.lock();
        let Some(handler) = irq_handler.get_mut(irq as usize) else {
            return;
        };

        *handler = Some(handler_fn);
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
