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
use crate::{GateType, GdtEntry, GlobalDescriptorTable, IdtEntry, InterruptDescriptorTable, serial_println, VirtualAddress};
use crate::arch_x86_64::idt::{SegmentSelector, TableSelect};

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

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        let mut idt_entry = IdtEntry::new();

        idt_entry.set_type_attributes(true, CpuPrivilegeLevel::RING0, GateType::InterruptGate);
        idt_entry.set_segment_selector(
            SegmentSelector::new(1, TableSelect::GDT, CpuPrivilegeLevel::RING0)
        );
        idt_entry.set_gate(VirtualAddress::from_ptr(&test_fn));

        idt.add_entry_range(idt_entry, 0..255);


        idt
    };
}

pub fn set_up_gdt() {
    GDT.submit_entries().load();
}

#[no_mangle]
#[cfg(feature = "abi_x86_interrupt")]
extern "x86-interrupt" fn test_fn() -> ! {

    serial_println!("Function ran");

    loop {}
}



pub fn set_up_idt() {

    IDT.submit_entries().load();

    unsafe {
        asm!("int 0x03");
    }
}