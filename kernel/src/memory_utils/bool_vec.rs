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
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

use core::ptr;
use crate::bitset::BitSet;
use crate::error_utils::QuantumError;

pub struct BoolVec<'a> {
    buffer: &'a mut [u8]
}

impl<'a> BoolVec<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer
        }
    }

    pub fn transfer_expand(&mut self, buffer: &'a mut [u8]) -> Result<(), QuantumError> {
        if buffer.len() >= self.buffer.len() {

            // transfer all bytes to new buffer
            for i in 0..self.buffer.len() {
                buffer[i] = self.buffer[i];
            }

            self.buffer = buffer;

            return Ok(());
        }

        Err(QuantumError::NoSpaceRemaining)
    }

    pub fn set_bit(&mut self, bit_id: usize, value: bool) -> Result<(), QuantumError> {
        if (bit_id / 8) < self.buffer.len() {
            self.buffer[bit_id / 8].set_bit((bit_id % 8) as u8, value);

            Ok(())
        } else {
            Err(QuantumError::OutOfRange)
        }

    }

    pub fn get_bit(&self, bit_id: usize) -> Option<bool> {
        if (bit_id / 8) < self.buffer.len() {
            Some(self.buffer[bit_id / 8].get_bit((bit_id % 8) as u8))
        } else {
            None
        }
    }

    pub fn find_first_of(&self, value: bool) -> Option<usize> {
        for i in 0..self.len() {
            if self.get_bit(i).unwrap_or(!value) == value {
                return Some(i);
            }
        }

        None
    }

    pub fn clear_entries(&mut self) {
        for i in &mut *self.buffer {
            *i = 0_u8;
        }
    }

    pub fn find_first_free(&self) -> Option<usize> {
        self.find_first_of(false)
    }

    pub fn find_first_set(&self) -> Option<usize> {
        self.find_first_of(true)
    }

    pub fn len(&self) -> usize {
        self.buffer.len() * 8
    }
}

#[cfg(test)]
mod test_case {
    use owo_colors::OwoColorize;
    use crate::memory_utils::bool_vec::BoolVec;

    #[test_case]
    pub fn construct_bool_type() {
        let mut limited_lifetime_buffer = [0_u8; 1024];
        let vector = BoolVec::new(&mut limited_lifetime_buffer);
    }

    #[test_case]
    pub fn set_bits_in_bool_type() {
        let mut limited_lifetime_buffer = [0_u8; 1024];
        let mut vector = BoolVec::new(&mut limited_lifetime_buffer);

        vector.set_bit(0, true).unwrap();
    }

    #[test_case]
    pub fn get_zero_value() {
        let mut limited_lifetime_buffer = [0_u8; 1024];
        let vector = BoolVec::new(&mut limited_lifetime_buffer);

        assert_eq!(vector.get_bit(0).unwrap(), false);
    }

    #[test_case]
    pub fn set_and_get_values() {
        let mut limited_lifetime_buffer = [0_u8; 1024];
        let mut vector = BoolVec::new(&mut limited_lifetime_buffer);


        for i in (2..41).step_by(3) {
            vector.set_bit(i, i % 2 == 0).unwrap();
        }

        for i in (2..41).step_by(3) {
            assert_eq!(vector.get_bit(i).unwrap(), i % 2 == 0);
        }

        vector.set_bit(1, true).unwrap();

        assert_eq!(vector.find_first_free().unwrap(), 0);
        assert_eq!(vector.find_first_set().unwrap(), 1);
    }

    #[test_case]
    pub fn expand_buffer_into_new() {
        let mut limited_lifetime_buffer = [0_u8; 1024];
        let mut vector = BoolVec::new(&mut limited_lifetime_buffer);

        for i in (2..41).step_by(3) {
            vector.set_bit(i, i % 2 == 0).unwrap();
        }

        for i in (2..41).step_by(3) {
            assert_eq!(vector.get_bit(i).unwrap(), i % 2 == 0);
        }

        vector.set_bit(1, true).unwrap();

        assert_eq!(vector.find_first_free().unwrap(), 0);
        assert_eq!(vector.find_first_set().unwrap(), 1);

        let mut new_limited_lifetime_buffer = [0_u8; 2048];
        vector.transfer_expand(&mut new_limited_lifetime_buffer).unwrap();

        for i in (2..41).step_by(3) {
            assert_eq!(vector.get_bit(i).unwrap(), i % 2 == 0);
        }

        assert_eq!(vector.find_first_free().unwrap(), 0);
        assert_eq!(vector.find_first_set().unwrap(), 1);
    }

    #[test_case]
    pub fn test_finding_free_and_set() {
        let mut limited_lifetime_buffer = [0_u8; 1024];
        let mut vector = BoolVec::new(&mut limited_lifetime_buffer);

        for i in 0..100 {
            vector.set_bit(i, true).unwrap();

            assert_eq!(vector.find_first_free().unwrap(), i + 1);
            assert_eq!(vector.find_first_set().unwrap(), 0);
        }

        vector.clear_entries();

        vector.set_bit(100, true).unwrap();

        assert_eq!(vector.find_first_set().unwrap(), 100);

        for i in 100..0 {
            vector.set_bit(i, false).unwrap();
            assert_eq!(vector.find_first_set().unwrap(), i);
        }

    }

}