/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
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

use core::arch::asm;
use crate::bios_println;
use crate::cpu_regs::{Regs16};

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum TextModeColor {
    Black        = 0b0000,
    Blue         = 0b0001,
    Green        = 0b0010,
    Cyan         = 0b0011,
    Red          = 0b0100,
    Magenta      = 0b0101,
    Brown        = 0b0110,
    LightGrey    = 0b0111,
    DarKGrey     = 0b1000,
    LightBlue    = 0b1001,
    LightGreen   = 0b1010,
    LightCyan    = 0b1011,
    LightRed     = 0b1100,
    LightMagenta = 0b1101,
    Yellow       = 0b1110,
    White        = 0b1111,
}

#[derive(PartialEq)]
pub enum BiosIntStatus {
    UnsupportedFunction,
    InvalidCommand,
    CommandFailed,

    SuccessfulCommand
}

impl BiosIntStatus {
    pub fn did_succeed(&self) -> bool {
        *self == BiosIntStatus::SuccessfulCommand
    }

    pub fn did_fail(&self) -> bool {
        *self != BiosIntStatus::SuccessfulCommand
    }

    pub fn unwrap(&self) {
        if self.did_succeed() {
            return;
        }

        panic!("unwrap() BiosInt Failed to execute command successfully!");
    }

    pub fn run_on_fail<F: Fn()>(&self, runnable: F) {
        if self.did_succeed() {
            return;
        }


        runnable();
    }
}

pub struct BiosInt {
    interrupt_number: u8,
    command: u8,
    flags: Regs16,

    result: u8
}

impl BiosInt {
    fn blank() -> Self {
        Self {
            interrupt_number: 0,
            command: 0,
            flags: Regs16::new(),
            result: 0,
        }
    }

    pub fn write_character(character: u8, page_number: u8, background_color: TextModeColor, foreground_color: TextModeColor) -> Self {
        let total_color = (foreground_color as u8) | ((background_color as u8) << 4);
        let mut regs = Regs16::new();

        regs.bl = total_color;
        regs.bh = page_number;
        regs.al = character;
        regs.cx = 0x1_u16;

        Self {
            interrupt_number: 0x10,
            command: 0x0E,
            flags: regs,
            ..Self::blank()
        }
    }

    pub fn read_vbe_info(struct_ptr: *mut u8) -> Self {
        let mut regs = Regs16::new();

        regs.di = struct_ptr as u16;

        Self {
            interrupt_number: 0x10,
            command: 0x4F,
            flags: regs,
            ..Self::blank()
        }
    }

    pub fn read_disk_with_packet(drive_number: u8, disk_packet: *mut u8) -> Self {
        let mut regs = Regs16::new();

        regs.dx = drive_number as u16;
        regs.si = disk_packet as u16;

        Self {
            interrupt_number: 0x13,
            command: 0x42,
            flags: regs,
            ..Self::blank()
        }
    }

    pub fn write_disk_with_packet(drive_number: u8, disk_packet: *mut u8) -> Self {
        let mut regs = Regs16::new();

        regs.dx = drive_number as u16;
        regs.si = disk_packet as u16;

        Self {
            interrupt_number: 0x13,
            command: 0x43,
            flags: regs,
            ..Self::blank()
        }
    }

    unsafe fn x10_int_dispatcher(&self) -> u8 {
        let res;

        asm!(
            "int 0x10",
            inout("ah") self.command => res,
            in("bl") self.flags.bl,
            in("bh") self.flags.bh,
            in("cx") self.flags.cx,
            in("al") self.flags.al,
            in("di") self.flags.di,
            clobber_abi("system")
        );

        res
    }

    unsafe fn x13_int_dispatcher(&self) -> u8 {
        let res;

        asm!(
            "push si",
            "mov si, {x:x}",
            "int 0x13",
            "pop si",
            x = in(reg) self.flags.si,
            inout("ah") self.command => res,
            in("dx") self.flags.dx,
            clobber_abi("system")
        );

        res
    }

    pub unsafe fn execute_interrupt(&self) -> BiosIntStatus {
        let res = match self.interrupt_number {
            0x10 => self.x10_int_dispatcher(),
            0x13 => self.x13_int_dispatcher(),

            _ => panic!("TODO: Not implemented bios int called!"),
        };

        match res {
            0x80 => BiosIntStatus::InvalidCommand,
            0x86 => BiosIntStatus::UnsupportedFunction,
            0x01 => BiosIntStatus::CommandFailed,

            _    => BiosIntStatus::SuccessfulCommand
        }
    }


}