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
use crate::{memory::VirtualAddress, serial_println};
use crate::arch_x86_64::idt::*;
use crate::arch_x86_64::gdt::*;


pub mod isr;
pub mod gdt;
pub mod tables;
pub mod idt;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum CpuPrivilegeLevel {
    RING0 = 0b00,
    RING1 = 0b01,
    RING2 = 0b10,
    RING3 = 0b11,
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

pub struct Idt(
    pub [IdtEntry; 16]
);

impl Idt {
    pub fn new() -> Self {
        Idt([IdtEntry::new(); 16])
    }


    pub fn load(&self) {
        let rg = IdtRegister { bytes: 16 * 16, ptr: (VirtualAddress::from_ptr(self)) };

        rg.load();

    }
}


lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();

        let mut idt_entry = IdtEntry::new();

        idt_entry.set_type_attributes(true, CpuPrivilegeLevel::RING0, GateType::InterruptGate);
        idt_entry.set_segment_selector(
            SegmentSelector::new(0, TableSelect::GDT, CpuPrivilegeLevel::RING0)
        );

        idt_entry.set_gate(test_fn);

        idt.0[2] = idt_entry;

        idt
    };
}

pub fn set_up_gdt() {
    //GDT.submit_entries().load();
}


extern "x86-interrupt" fn test_fn(isf: InterruptStackFrame, e: u64) {
    serial_println!("Function ran");

    loop {};
}

pub fn set_up_idt() {

    IDT.load();

    serial_println!("U64 max = {}", u64::MAX);

    serial_println!("Size of entry: {}", size_of::<IdtEntry>());

    unsafe {
        asm!("int 0x03");
    }
}

