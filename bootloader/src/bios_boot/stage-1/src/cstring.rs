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

use core::fmt;
use core::fmt::Formatter;

pub struct CStringOwned {
    owned_data_ptr: *const u8,
    len: usize,
}

impl CStringOwned {
    pub unsafe fn from_ptr(bytes: *const u8, len: usize) -> Self {
        Self {
            owned_data_ptr: bytes,
            len,
        }
    }
    
    pub fn from_static_bytes(bytes: &'static [u8]) -> Self {
        unsafe { Self::from_ptr(bytes.as_ptr(), bytes.len()) }
    }
}

impl Default for CStringOwned {
    fn default() -> Self {
        Self {
            owned_data_ptr: &[0u8; 0] as *const u8,
            len: 0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct CStringRef<'a> {
    contents: &'a [u8],
    len: usize,
}

impl<'a> CStringRef<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Self {
            contents: bytes,
            len: bytes.len(),
        }
    }

    pub unsafe fn from_ptr(ptr: *mut u8) -> Self {
        let mut size = 0;
        loop {
            let ptr_value = *ptr.add(size);

            if ptr_value == b'\0' || !ptr_value.is_ascii() {
                break;
            }

            size += 1;
        }

        let array = core::slice::from_raw_parts(ptr, size);

        Self {
            contents: array,
            len: size,
        }
    }
}

impl<'a> fmt::Display for CStringRef<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for i in self.contents {
            write!(f, "{}", *i as char)?;
        }

        Ok(())
    }
}

impl fmt::Display for CStringOwned {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for i in 0..self.len {
            let data = unsafe { *self.owned_data_ptr.add(i) };
            write!(f, "{}", data as char)?;
        }

        Ok(())
    }
}
