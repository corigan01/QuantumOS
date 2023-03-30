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

impl Default for Regs16 {
    fn default() -> Self {
        Self::new()
    }
}
