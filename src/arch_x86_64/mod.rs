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
use core::mem::size_of;
use lazy_static::lazy_static;
use x86_64::instructions::segmentation;
use crate::{memory::VirtualAddress, serial_println};
use crate::arch_x86_64::idt::*;
use crate::arch_x86_64::gdt::*;
use crate::general_function_to_interrupt_ne;
use crate::general_function_to_interrupt_de;
use crate::interrupt_wrapper;
use crate::attach_interrupt;


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

lazy_static! {
    static ref GDT: GlobalDescriptorTable = {
        let mut gdt = GlobalDescriptorTable::new();

        gdt.add_entry(GdtEntry::new()).unwrap();
        gdt.add_entry(GdtEntry::new_raw(0, 0xFFFFFFFF, 0x9A, 0xCF)).unwrap();
        gdt.add_entry(GdtEntry::new_raw(0, 0xFFFFFFFF, 0x92, 0xCF)).unwrap();

        gdt
    };
}

fn divide_by_zero_handler(i_frame: InterruptFrame, int_n: u8, error_code: Option<u64>) {
    serial_println!("EXCEPTION: DIVIDE BY ZERO {}", int_n);
}

lazy_static! {
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();

        attach_interrupt!(idt, divide_by_zero_handler, 0);


        idt
    };
}

pub fn init_idt() {
    IDT.submit_entries().unwrap().load();

    serial_println!("Segmentation: {:#?}", segmentation::cs());

    divide_by_zero();
}

pub fn divide_by_zero() {
    unsafe {
        asm!("int $0x0");
    }
}


