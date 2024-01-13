/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2024 Gavin Kellam

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

use quantum_lib::x86_64::io::port::IOPort;

// -- CONSTs pulled from OSDev Wiki, to be abstracted later
// -- Link: https://wiki.osdev.org/8259_PIC
const PIC_MAIN: u16 = 0x20;
const PIC_SECOND: u16 = 0xA0;
const PIC_MAIN_COMMAND: u16 = PIC_MAIN;
const PIC_SECOND_COMMAND: u16 = PIC_SECOND;
const PIC_MAIN_DATA: u16 = PIC_MAIN + 1;
const PIC_SECOND_DATA: u16 = PIC_SECOND + 1;

const PIC_EOI: u8 = 0x20;

const ICW1_ICW4: u8 = 0x01;
const ICW1_SINGLE: u8 = 0x02;
const ICW1_INTERVAL4: u8 = 0x04;
const ICW1_LEVEL: u8 = 0x08;
const ICW1_INIT: u8 = 0x10;

const ICW4_8086: u8 = 0x01;
const ICW4_AUTO: u8 = 0x02;
const ICW4_BUF_SECOND: u8 = 0x08;
const ICW4_BUF_MAIN: u8 = 0x0C;
const ICW4_SFNM: u8 = 0x10;

const PIC_SECOND_VECTOR_OFFSET: u8 = 0x04;
const PIC_SECOND_CASCADE_IDENTIY: u8 = 0x02;

#[inline]
pub unsafe fn pic_eoi(interrupt_id: u8) {
    assert!(interrupt_id <= 16, "IRQ must be lower then 16");

    if interrupt_id >= 8 {
        IOPort::new(PIC_SECOND_COMMAND).write_u8(PIC_EOI)
    }

    IOPort::new(PIC_MAIN_COMMAND).write_u8(PIC_EOI)
}

#[inline]
pub fn wait_bus() {
    for _ in 0..16 {
        unsafe {
            IOPort::new(PIC_MAIN).read_u8();
        }
    }
}

pub unsafe fn pic_init(idt_offset: u8) {
    assert!(
        idt_offset + 8 <= 255,
        "IDT offest must fit 16 interrupt vectors into the 256 range"
    );

    let pic_main_masks = IOPort::new(PIC_MAIN_DATA).read_u8();
    let pic_second_masks = IOPort::new(PIC_SECOND_DATA).read_u8();

    // Main init
    wait_bus();
    IOPort::new(PIC_MAIN_COMMAND).write_u8(ICW1_INIT | ICW1_ICW4);
    wait_bus();
    IOPort::new(PIC_SECOND_COMMAND).write_u8(ICW1_INIT | ICW1_ICW4);
    wait_bus();
    IOPort::new(PIC_MAIN_DATA).write_u8(idt_offset);
    wait_bus();
    IOPort::new(PIC_SECOND_DATA).write_u8(idt_offset + 8);
    wait_bus();
    IOPort::new(PIC_MAIN_DATA).write_u8(PIC_SECOND_VECTOR_OFFSET);
    wait_bus();
    IOPort::new(PIC_SECOND_DATA).write_u8(PIC_SECOND_CASCADE_IDENTIY);
    wait_bus();
    IOPort::new(PIC_MAIN_DATA).write_u8(ICW4_8086);
    wait_bus();
    IOPort::new(PIC_SECOND_DATA).write_u8(ICW4_8086);
    wait_bus();

    // Restore masks
    IOPort::new(PIC_MAIN_DATA).write_u8(pic_main_masks);
    IOPort::new(PIC_SECOND_DATA).write_u8(pic_second_masks);
}

pub unsafe fn pic_disable() {
    IOPort::new(PIC_MAIN_DATA).write_u8(0xFF);
    IOPort::new(PIC_SECOND_DATA).write_u8(0xFF);
}

pub unsafe fn pic_set_irq_mask(interrupt: u8) {
    let (pic, interrupt) = if interrupt >= 8 {
        (PIC_SECOND_DATA, interrupt - 8)
    } else {
        (PIC_MAIN_DATA, interrupt)
    };

    let pic = IOPort::new(pic);
    let mask = pic.read_u8() | (1 << interrupt);
    pic.write_u8(mask);
}

pub unsafe fn pic_clear_irq_mask(interrupt: u8) {
    let (pic, interrupt) = if interrupt >= 8 {
        (PIC_SECOND_DATA, interrupt - 8)
    } else {
        (PIC_MAIN_DATA, interrupt)
    };

    let pic = IOPort::new(pic);
    let mask = pic.read_u8() & !(1 << interrupt);
    pic.write_u8(mask);
}
