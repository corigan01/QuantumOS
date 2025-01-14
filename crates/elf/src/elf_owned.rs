/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

extern crate alloc;
use alloc::vec::Vec;

use crate::Elf;

/// An owned version of an Elf file's bytes
#[derive(Clone)]
pub struct ElfOwned {
    array: Vec<u64>,
}

impl ElfOwned {
    pub fn new_from_slice(elf: &[u8]) -> ElfOwned {
        let mut inner_array: Vec<u64> = Vec::with_capacity((elf.len() - 1) / 8 + 1);
        let ptr: *mut u8 = inner_array.as_mut_ptr().cast();

        let aligned_u8_slice = unsafe { core::slice::from_raw_parts_mut(ptr, elf.len()) };
        aligned_u8_slice.copy_from_slice(elf);

        Self { array: inner_array }
    }

    pub fn as_bytes<'a>(&'a self) -> &'a [u8] {
        unsafe {
            core::slice::from_raw_parts(self.array.as_ptr() as *const u8, self.array.len() * 8)
        }
    }

    pub fn elf<'a>(&'a self) -> Elf<'a> {
        Elf {
            elf_file: self.as_bytes(),
        }
    }
}
