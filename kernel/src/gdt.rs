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

// GDT
use core::{arch::asm, mem::size_of};

static mut LONG_MODE_GDT: GdtLongMode = GdtLongMode::new();

#[repr(C)]
pub struct GdtLongMode {
    zero: u64,
    code: u64,
    data: u64,
}

impl GdtLongMode {
    const fn new() -> Self {
        let common_flags = {
            (1 << 44) // user segment
                | (1 << 47) // present
                | (1 << 41) // writable
                | (1 << 40) // accessed (to avoid changes by the CPU)
        };
        Self {
            zero: 0,
            code: common_flags | (1 << 43) | (1 << 53), // executable and long mode
            data: common_flags,
        }
    }

    pub fn load(&'static self) {
        let pointer = GdtPointer {
            limit: (3 * size_of::<u64>() - 1) as u16,
            base: self,
        };

        unsafe {
            asm!("lgdt [{}]", in(reg) &pointer);
        }
    }
}

#[repr(C, packed(2))]
pub struct GdtPointer {
    /// Size of the DT.
    pub limit: u16,
    /// Pointer to the memory region containing the DT.
    pub base: *const GdtLongMode,
}
