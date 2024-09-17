/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

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

use core::{
    arch::asm,
    ops::{Add, Sub},
};

#[derive(Clone, Copy)]
pub struct IOPort(u16);

impl IOPort {
    /// # New
    /// Crate a new x86 io port struct.
    pub const fn new(port: u16) -> Self {
        Self(port)
    }

    /// # Read Byte
    /// Read a byte from the cpu io bus.
    #[inline(always)]
    pub unsafe fn read_byte(self) -> u8 {
        let mut port_value;

        asm!("in al, dx", out("al") port_value, in("dx") self.0, options(nomem, nostack, preserves_flags));
        return port_value;
    }

    /// # Write Byte
    /// Write a byte to the cpu io bus.
    pub unsafe fn write_byte(self, byte: u8) {
        asm!("out dx, al", in("dx") self.0, in("al") byte, options(nomem, nostack, preserves_flags));
    }

    /// # Read Word
    /// Read a word from the cpu io bus.
    pub unsafe fn read_word(self) -> u16 {
        let mut port_value;

        asm!("in ax, dx", out("ax") port_value, in("dx") self.0, options(nomem, nostack, preserves_flags));
        return port_value;
    }

    /// # Write Word
    /// Writes a word to the cpu io bus.
    pub unsafe fn write_word(self, word: u16) {
        asm!("out dx, ax", in("dx") self.0, in("ax") word, options(nomem, nostack, preserves_flags));
    }
}

impl Add<u16> for IOPort {
    type Output = Self;

    fn add(self, rhs: u16) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<u16> for IOPort {
    type Output = Self;

    fn sub(self, rhs: u16) -> Self::Output {
        Self(self.0 - rhs)
    }
}
