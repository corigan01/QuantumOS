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

use core::arch::asm;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::arch_x86_64::gdt::*;
use crate::arch_x86_64::idt::*;
use crate::attach_interrupt;
use crate::remove_interrupt;
use crate::{serial_println, serial_print};

pub mod isr;
pub mod gdt;
pub mod tables;
pub mod idt;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum CpuPrivilegeLevel {
    Ring0 = 0b00,
    Ring1 = 0b01,
    Ring2 = 0b10,
    Ring3 = 0b11,
}



fn test_interrupt_dev0(iframe: InterruptFrame, interrupt: u8, error: Option<u64>) {
    serial_println!("OK");
}

lazy_static! {
    pub static ref GLOBAL_DT: GlobalDescriptorTable = {
        let mut gdt = GlobalDescriptorTable::new();

        serial_print!("Checking GDT ... ");

        //gdt.add_entry(GdtEntry::new()).unwrap();
        //gdt.add_entry(GdtEntry::new_raw(0, 0xFFFFFFFF, 0x9A, 0xCF)).unwrap();
        //gdt.add_entry(GdtEntry::new_raw(0, 0xFFFFFFFF, 0x92, 0xCF)).unwrap();

        serial_println!("OK");

        gdt
    };
}

lazy_static! {
    pub static ref INTERRUPT_DT: Mutex<idt::Idt> = {
        let mut idt = idt::Idt::new();

        serial_print!("Checking IDT ... ");

        attach_interrupt!(idt, test_interrupt_dev0, 0);

        idt.submit_entries().expect("Unable to load IDT!").load();

        // test the handler
        unsafe {
            asm!("int $0x00");
        }

        remove_interrupt!(idt, 0);

        idt.submit_entries().expect("Unable to load IDT!").load();

        Mutex::new(idt)
    };
}



