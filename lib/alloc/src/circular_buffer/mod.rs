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

use core::fmt::Write;
use core::ops::Index;
use core::str::Utf8Error;
use crate::heap::{AllocatorAPI, GlobalAlloc};
use crate::vec::Vec;

pub struct CircularBuffer<Type, Alloc: AllocatorAPI = GlobalAlloc> {
    buffer: Vec<Type, Alloc>
}

impl<Type, Alloc: AllocatorAPI> CircularBuffer<Type, Alloc> {
    pub fn new(cap: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(cap)
        }
    }

    pub fn expand(&mut self, widen: usize) {
        self.buffer.reserve(widen);
    }

    pub fn push_back(&mut self, value: Type) {
        if self.buffer.len() == self.buffer.capacity() {
            self.buffer.remove(0);
        }

        self.buffer.push(value);
    }

    pub fn push_front(&mut self, value: Type) {
        if self.buffer.len() == self.buffer.capacity() {
            self.buffer.pop();
        }

        self.buffer.insert(0, value);
    }

    pub fn pop_back(&mut self) -> Option<Type> {
        self.buffer.pop()
    }

    pub fn pop_front(&mut self) -> Option<Type> {
        if self.buffer.len() == 0 {
            return None;
        }

        Some(self.buffer.remove(0))
    }

    pub fn clear(&mut self) {
        while let Some(_) = self.buffer.pop() {}
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn to_vec(&self) -> Vec<Type, Alloc>
        where Type: Clone {
        self.buffer.clone()
    }

    pub fn as_slice(&self) -> &[Type] {
        self.buffer.as_slice()
    }
}

impl<Alloc: AllocatorAPI> CircularBuffer<u8, Alloc> {
    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        core::str::from_utf8(self.as_slice())
    }

    pub unsafe fn as_str_unchecked(&self) -> &str {
        core::str::from_utf8_unchecked(self.as_slice())
    }
}

impl<Type, Alloc: AllocatorAPI> Index<usize> for CircularBuffer<Type, Alloc> {
    type Output = Type;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer.get(index).expect("Index out of Bounds!")
    }
}

impl<Alloc: AllocatorAPI> Write for CircularBuffer<u8, Alloc> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.push_back(byte);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::heap::set_example_allocator;

    #[test]
    fn test_push_around_buffer() {
        set_example_allocator(4096);
        let mut buf: CircularBuffer<i32> = CircularBuffer::new(4);

        buf.push_back(0);
        buf.push_back(1);
        buf.push_back(2);
        buf.push_back(3);
        buf.push_back(4);

        assert_eq!(buf[0], 1);
        assert_eq!(buf[1], 2);
        assert_eq!(buf[2], 3);
        assert_eq!(buf[3], 4);
    }

    #[test]
    fn limited_string() {
        set_example_allocator(4096);
        let mut buf: CircularBuffer<u8> = CircularBuffer::new(5);

        write!(buf, "Hello World!").unwrap();

        assert_eq!(buf.as_str(), Ok("orld!"));
    }

    #[test]
    fn clear_and_expand() {
        set_example_allocator(4096);
        let mut buf: CircularBuffer<u8> = CircularBuffer::new(5);
        write!(buf, "Hello World!").unwrap();
        assert_eq!(buf.as_str(), Ok("orld!"));

        buf.clear();
        assert_eq!(buf.len(), 0);

        buf.expand(1);
        write!(buf, "Hello World!").unwrap();
        assert_eq!(buf.as_str(), Ok("World!"));
    }


}