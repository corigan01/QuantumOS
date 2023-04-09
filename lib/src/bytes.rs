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

use core::fmt::{Debug, Display, Formatter};

#[derive(Clone, Copy, PartialOrd, PartialEq, Default, Debug)]
pub struct Bytes {
    bytes: u128,
}

impl Bytes {
    pub fn new() -> Self {
        Self { bytes: 0 }
    }
}

impl Display for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let deviser = if self.bytes > 1024 {
            (1024, "Kib")
        } else if self.bytes > (1024 * 1024) {
            (1024 * 1024, "Mib")
        } else if self.bytes > (1024 * 1024 * 1024) {
            (1024 * 1024 * 1024, "Gib")
        } else {
            (1, "bytes")
        };

        let norm_bytes = self.bytes / deviser.0;
        let symb = deviser.1;

        write!(f, "{} {}", norm_bytes, symb)?;

        Ok(())
    }
}

macro_rules! from_all_types {
    ($($t:ty)*) => ($(
        impl From<$t> for Bytes {
            fn from(value: $t) -> Self {
                Bytes {
                    bytes: value as u128
                }
            }
        }
    )*)
}

from_all_types! {usize u8 u16 u32 u64 u128}
