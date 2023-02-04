/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

use core::ptr;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::error_utils::QuantumError;
use crate::memory::VirtualAddress;

const TOTAL_BYTES: usize = 1024 * 50; // 50Kib for preliminary allocations
static BYTES: [u8; TOTAL_BYTES] = [0; TOTAL_BYTES];

pub struct SimpleAllocator {
    buffer: VirtualAddress,
    total_size: usize,
    used: usize
}

lazy_static! {
    pub static ref INIT_ALLOC: Mutex<SimpleAllocator> = {
        Mutex::new(
            SimpleAllocator::new(VirtualAddress::from_ptr(BYTES.as_ptr()), BYTES.len())
        )
    };
}

impl SimpleAllocator {
    pub fn new(buffer: VirtualAddress, size: usize) -> Self {
        Self {
            buffer,
            total_size: size,
            used: 0
        }
    }

    pub fn alloc(&mut self, bytes: usize) -> Result<*mut [u8], QuantumError> {
        if self.used >= self.total_size {
            return Err(QuantumError::NoSpaceRemaining);
        }

        let ptr = unsafe { (self.buffer.as_mut_ptr() as *mut u8).add(self.used) };

        self.used += bytes;

        Ok(ptr::slice_from_raw_parts_mut(ptr, bytes))
    }

    pub fn alloc_aligned(&mut self, bytes: usize) -> Result<*mut [u8], QuantumError> {
        let ptr = self.buffer.as_u64();
        let alignment_offset = (ptr & (u64::MAX - (0x1000 - 1))) + 0x1000;
        let offset_before_ptr = (alignment_offset - ptr) as usize;

        if self.used + offset_before_ptr >= self.total_size {
            return Err(QuantumError::NoSpaceRemaining);
        }

        let ptr = unsafe {
            (self.buffer.as_mut_ptr() as *mut u8)
                .add(offset_before_ptr)
                .add(self.used)
        };

        self.used += bytes + offset_before_ptr;

        Ok(ptr::slice_from_raw_parts_mut(ptr, bytes))
    }

    pub fn remaining_capacity(&self) -> usize {
        self.total_size - self.used
    }
}

#[cfg(test)]
pub mod test {
    use crate::memory::init_alloc::INIT_ALLOC;
    use crate::memory::VirtualAddress;

    #[test_case]
    pub fn test_case_alloc() {
        let mut alloc = INIT_ALLOC.lock();

        assert_ne!(alloc.buffer.as_u64(), 0);
        assert_ne!(VirtualAddress::from_ptr(alloc.alloc(10).unwrap() as *const u8).as_u64(), 0);
        assert_eq!(alloc.used, 10);
    }

    #[test_case]
    pub fn test_case_alloc_aligned() {
        let mut alloc = INIT_ALLOC.lock();

        assert!(
            (VirtualAddress::from_ptr(
                alloc.alloc_aligned(4096).unwrap() as *const u8
            ).as_u64()) & (0x1000 - 1) > 0
        );

        assert!(
            (VirtualAddress::from_ptr(
                alloc.alloc_aligned(234).unwrap() as *const u8
            ).as_u64()) & (0x1000 - 1) > 0
        );

        assert!(
            (VirtualAddress::from_ptr(
                alloc.alloc_aligned(1).unwrap() as *const u8
            ).as_u64()) & (0x1000 - 1) > 0
        );
    }
}