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

use crate::error::BootloaderError;
use core::mem::size_of;
use lazy_static::lazy_static;
use spin::Mutex;

pub struct AllocMemoryRegion {
    memory_location_ptr: *mut u8,
    memory_alloc_size: usize,
    used_space: usize,
}

pub struct MemoryAllocationEntry {
    magic: u64,
    allocation_size: u32,
}

unsafe impl Send for AllocMemoryRegion {}
unsafe impl Sync for AllocMemoryRegion {}

lazy_static! {
    static ref TEMP_ALLOC: Mutex<Option<AllocMemoryRegion>> = { Mutex::new(None) };
}

impl AllocMemoryRegion {
    pub fn new(ptr: *mut u8, size: usize) -> Result<Self, BootloaderError> {
        if size < size_of::<MemoryAllocationEntry>() * 2 + 1 {
            return Err(BootloaderError::NotEnoughMemory);
        }

        Ok(Self {
            memory_location_ptr: ptr,
            memory_alloc_size: size,
            used_space: 0,
        })
    }

    pub fn get_remaining_space(&self) -> usize {
        self.memory_alloc_size - self.used_space
    }

    pub fn allocate<'a>(&self, size: usize) -> Result<&'a mut [u8], BootloaderError> {
        todo!()
    }
}
