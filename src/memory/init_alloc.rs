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

const TOTAL_BYTES: usize = 1024 * 10;

pub struct SimpleAllocator {
    buffer: [u8; TOTAL_BYTES],
    used: usize
}

lazy_static! {
    pub static ref INIT_ALLOC: Mutex<SimpleAllocator> = {
        Mutex::new(SimpleAllocator::new())
    };
}

impl SimpleAllocator {
    pub fn new() -> Self {
        Self {
            buffer: [0_u8; TOTAL_BYTES],
            used: 0
        }
    }

    pub fn alloc(&mut self, bytes: usize) -> Result<*mut [u8], QuantumError> {
        self.used += bytes;

        if self.used >= TOTAL_BYTES {
            return Err(QuantumError::NoSpaceRemaining);
        }

        Ok(ptr::slice_from_raw_parts_mut(self.buffer.as_mut_ptr(), self.used))
    }


}