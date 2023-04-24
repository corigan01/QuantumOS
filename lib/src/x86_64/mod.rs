/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
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

pub mod bios_call;
pub mod interrupts;
pub mod io;
pub mod paging;
pub mod registers;

/// Enumeration of possible privilege levels (rings) in x86 and x86_64 architectures.
pub enum PrivlLevel {
    /// Ring 0, the most privileged level, typically reserved for the kernel.
    Ring0,
    /// Ring 1, a less privileged level, typically used for device drivers.
    Ring1,
    /// Ring 2, a less privileged level, typically used for user-defined code with elevated privileges.
    Ring2,
    /// Ring 3, the least privileged level, typically used for user space applications.
    Ring3,
}

impl PrivlLevel {
    /// Creates a new `PrivlLevel` from a `usize` value.
    /// Returns `Some(PrivlLevel)` if `value` is between 0 and 3 inclusive, otherwise returns `None`.
    pub fn new_from_usize(value: usize) -> Option<Self> {
        match value {
            0 => Some(Self::Ring0),
            1 => Some(Self::Ring1),
            2 => Some(Self::Ring2),
            3 => Some(Self::Ring3),
            _ => None,
        }
    }

    /// Returns the `usize` representation of the `PrivlLevel`.
    pub fn to_usize(&self) -> usize {
        match self {
            Self::Ring0 => 0,
            Self::Ring1 => 1,
            Self::Ring2 => 2,
            Self::Ring3 => 3,
        }
    }
}

pub struct CPU {}

impl CPU {
    pub fn halt() -> ! {
        loop {}
    }
}
