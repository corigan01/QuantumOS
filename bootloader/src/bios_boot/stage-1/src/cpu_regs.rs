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

#[repr(u16)]
pub enum EFlagsStates {
    Carry = 0x00,
    Parity = 0x02,
    Auxiliary = 0x04,
    Zero = 0x06,
    Sign = 0x07,
    Trap = 0x08,
    InterruptEnable = 0x09,
    Direction = 0x0A,
    Overflow = 0x0B,
    IOP0 = 0x0C,
    IOP1 = 0x0D,
    NestedTask = 0x0E,
    Resume = 0x0F,
    // Higher flags unsupported
}

#[repr(C)]
pub struct EFlags {
    low: u16,
    high: u16,
}

impl EFlags {
    pub fn new() -> Self {
        let mut flags = Self::new_zero();
        flags.update_flags();

        flags
    }

    pub fn new_zero() -> Self {
        Self { low: 0, high: 0 }
    }

    pub fn update_flags(&mut self) {
        todo!()
    }

    pub fn check_flag_status(&self, flag: EFlagsStates) -> bool {
        self.low & (flag as u16) > 0
    }
}

#[repr(C, packed)]
pub struct Regs16 {
    pub di: u16,
    pub si: u16,
    pub bp: u16,
    pub sp: u16,
    pub bx: u16,
    pub dx: u16,
    pub cx: u16,
    pub ax: u16,

    pub al: u8,
    pub bl: u8,
    pub cl: u8,
    pub dl: u8,
    pub ah: u8,
    pub bh: u8,
    pub ch: u8,
    pub dh: u8,

    pub gs: u16,
    pub fs: u16,
    pub es: u16,
    pub ds: u16,
    pub eflags: u16,
}

impl Regs16 {
    pub fn new() -> Self {
        Regs16 {
            di: 0,
            si: 0,
            bp: 0,
            sp: 0,
            bx: 0,
            dx: 0,
            cx: 0,
            ax: 0,

            al: 0,
            bl: 0,
            cl: 0,
            dl: 0,
            ah: 0,
            bh: 0,
            ch: 0,
            dh: 0,

            gs: 0,
            fs: 0,
            es: 0,
            ds: 0,
            eflags: 0,
        }
    }
}
