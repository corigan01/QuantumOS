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

use core::fmt::{Formatter, Write};
use core::ops::Deref;
use crate::heap::{AllocatorAPI, GlobalAlloc};
use crate::vec::Vec;

pub struct String<Alloc: AllocatorAPI = GlobalAlloc> {
    data: Vec<u8, Alloc>
}

impl String {
    pub const fn new() -> Self {
        Self {
            data: Vec::new()
        }
    }

    pub fn push(&mut self, c: char) {
        let mut tmp_buffer: [u8; 4] = [0; 4];

        let data_len = c.len_utf8();
        c.encode_utf8(&mut tmp_buffer);

        for i in 0..data_len {
            self.data.push(tmp_buffer[i]);
        }
    }

    pub fn push_str(&mut self, s: &str) {
        for i in s.bytes() {
            self.data.push(i);
        }
    }

    pub fn as_str(&self) -> &str {
        // We know its valid utf8 because we can only push valid utf8 into the array
        unsafe { core::str::from_utf8_unchecked(self.data.as_slice()) }
    }

}

impl PartialEq<&str> for String {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for String {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Deref for String {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Write for String {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.push_str(s);
        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.push(c);
        Ok(())
    }
}

impl<T> From<T> for String
    where T: core::fmt::Display {
    fn from(value: T) -> Self {
        let mut tmp = String::new();
        write!(&mut tmp, "{}", value).unwrap();

        tmp
    }
}

#[cfg(test)]
mod test {
    use crate::heap::set_example_allocator;
    use super::*;

    #[test]
    fn test_string_from_literal() {
        set_example_allocator(4096);

        let string = String::from("My String!");
        assert_eq!(string.as_str(), "My String!");
    }

    #[test]
    fn test_string_from_non_literal() {
        set_example_allocator(4096);

        let one_char = String::from('1');
        let one_u8 = String::from(1_u8);
        let one_usize = String::from(1_usize);
        let one_str = String::from("1");
        let one_i32 = String::from(1);
        let one_f32 = String::from(1_f32);

        assert_eq!(one_char.as_str(), "1");
        assert_eq!(one_u8.as_str(), "1");
        assert_eq!(one_usize.as_str(), "1");
        assert_eq!(one_str.as_str(), "1");
        assert_eq!(one_i32.as_str(), "1");
        assert_eq!(one_f32.as_str(), "1");
    }

    #[test]
    fn test_string_str_deref() {
        set_example_allocator(4096);

        let example = String::from("QuantumOS");

        assert_eq!(example.is_ascii(), true);
        assert_eq!(example.contains("OS"), true);
        assert_eq!(example.is_empty(), false);
        assert_eq!(example.as_str(), "QuantumOS");
    }

}