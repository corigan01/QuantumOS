/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

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

use crate::io::{io_wait, IOPort};

// Based on the OSDev.wiki 8259 PIC article
// TODO: Base this impl on documentation
//       for a more complete impl of the hardware.

const PIC_1_COMMAND: IOPort = IOPort::new(0x20);
const PIC_1_DATA: IOPort = IOPort::new(0x21);
const PIC_2_COMMAND: IOPort = IOPort::new(0xA0);
const PIC_2_DATA: IOPort = IOPort::new(0xA1);

const OCW_EOI: u8 = 1 << 5;

const CW1_ENABLE_CW4: u8 = 1 << 0;
const _CW1_SINGLE_MODE: u8 = 1 << 1;
const _CW1_INTERVAL_SELECT: u8 = 1 << 2;
const _CW1_LEVEL_TRIGGER: u8 = 1 << 3;
const CW1_INIT: u8 = 1 << 4;

const CW4_8086_MODE: u8 = 1 << 0;
const _CW4_AUTO: u8 = 1 << 1;
const _CW4_BUFFER_2_MODE: u8 = 1 << 3;
const _CW4_BUFFER_1_MODE: u8 = 0x0C;
const _CW4_DISABLE_SPECIAL_NESTED_MODE: u8 = 1 << 4;

pub unsafe fn pic_remap(vector_offset_1: u8, vector_offset_2: u8) {
    unsafe {
        let pic_1_mask = PIC_1_DATA.read_byte();
        let pic_2_mask = PIC_2_DATA.read_byte();

        // Init and enable CW4
        PIC_1_COMMAND.write_byte(CW1_INIT | CW1_ENABLE_CW4);
        io_wait();
        PIC_2_COMMAND.write_byte(CW1_INIT | CW1_ENABLE_CW4);
        io_wait();

        // Set Vector Offsets
        PIC_1_DATA.write_byte(vector_offset_1);
        io_wait();
        PIC_2_DATA.write_byte(vector_offset_2);
        io_wait();

        // Inform PIC1 that there is a PIC2
        PIC_1_DATA.write_byte(4);
        io_wait();
        PIC_2_DATA.write_byte(2);
        io_wait();

        // Change mode to 8086
        PIC_1_DATA.write_byte(CW4_8086_MODE);
        io_wait();
        PIC_2_DATA.write_byte(CW4_8086_MODE);
        io_wait();

        // Restore IRQ masks
        PIC_1_DATA.write_byte(pic_1_mask);
        io_wait();
        PIC_2_DATA.write_byte(pic_2_mask);
        io_wait();
    }
}

pub unsafe fn pic_eoi(irq: u8) {
    assert!(irq < 16, "Cannot have a IRQ larger then 16!");
    if irq >= 8 {
        PIC_2_COMMAND.write_byte(OCW_EOI);
    }
    PIC_1_COMMAND.write_byte(OCW_EOI);
}

pub unsafe fn pic_mask_irq(irq: u8) {
    assert!(irq < 16, "Cannot have a IRQ larger then 16!");
    let port = if irq < 8 { PIC_1_DATA } else { PIC_2_DATA };

    let new_mask = port.read_byte() | (1 << irq);
    port.write_byte(new_mask);
}

pub unsafe fn pic_unmask_irq(irq: u8) {
    assert!(irq < 16, "Cannot have a IRQ larger then 16!");
    let port = if irq < 8 { PIC_1_DATA } else { PIC_2_DATA };

    let new_mask = port.read_byte() & !(1 << irq);
    port.write_byte(new_mask);
}

pub unsafe fn pic_disable() {
    PIC_1_DATA.write_byte(0xFF);
    PIC_2_DATA.write_byte(0xFF);
}
