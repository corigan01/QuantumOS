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

pub struct HeaplessString<const SIZE: usize> {
    internal_buffer: [u8; SIZE],
}

impl<const SIZE: usize> HeaplessString<SIZE> {
    pub fn new() -> Self {
        Self {
            internal_buffer: [0u8; SIZE],
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() > SIZE {
            return None;
        }

        let mut tmp_buffer: [u8; SIZE] = [0; SIZE];

        for (i, byte) in bytes.iter().enumerate() {
            tmp_buffer[i] = *byte;
        }

        Some(Self {
            internal_buffer: tmp_buffer,
        })
    }

    pub fn from_str(string: &str) -> Option<Self> {
        Self::from_bytes(string.as_bytes())
    }

    pub fn get_str(&self) -> &str {
        return unsafe { core::str::from_utf8_unchecked(&self.internal_buffer) }.trim_end();
    }

    pub fn get_mut_str(&mut self) -> &str {
        return unsafe { core::str::from_utf8_unchecked_mut(&mut self.internal_buffer) }.trim_end();
    }

    pub fn fill_zero(&mut self) {
        self.internal_buffer.fill(0);
    }
}

impl<const SIZE: usize> Debug for HeaplessString<SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "HeaplessString {{")?;
        writeln!(f, "    total_size: {}", SIZE)?;
        writeln!(f, "    string: {}", self.get_str())?;
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl<const SIZE: usize> Display for HeaplessString<SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.get_str())
    }
}

impl<const SIZE: usize> PartialEq for HeaplessString<SIZE> {
    fn eq(&self, other: &Self) -> bool {
        if other.internal_buffer.len() != SIZE {
            return false;
        }

        other.internal_buffer == self.internal_buffer
    }
}
