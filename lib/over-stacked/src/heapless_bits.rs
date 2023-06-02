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

/*!
 * # Heapless Bits
 */

use crate::heapless_vector::{HeaplessVec, HeaplessVecErr};

/// # Heapless Bits
/// Bits aids in the control of bit level modification, this is often used to create
/// allocators or other applications that require chunks of memory or chunks of a resource
/// to be allocated. 
/// 
/// # Example Useage
/// ```rust
/// use over_stacked::heapless_bits::HeaplessBits;
/// 
/// fn main() {
///     let mut bit_map: HeaplessBits<10> = HeaplessBits::new();
/// 
///     bit_map.set_bit(10, true).unwrap();
/// 
///     assert_eq!(bit_map.get_bit(10).unwrap(), true);
/// } 
/// ```
/// 
/// # Stack Based Allocation
/// Stack based allocation has limitations as does all allocation methods. However,
/// stack based is often the least flexable. This is because the stack needs to be 
/// a known size at compile time. This limits us to having to set the amount of 
/// 'BYTES' we want to use for our allocations. 
/// 
/// This is often very useful in cases where a heap allocator isn't practical, 
/// and the need to something on the stack is a perfect choice. 
/// 
/// # Limitations
/// * Since we are limited to a stack allocation enviroment, this type is very slow 
/// to pass by value. This type *contains* its entire buffer, so its not able to be
/// easily transfered. 
/// 
/// * This can not allocate more memory! Even if an allocator is available, this type
/// will not allocate more memory. It is limited to the size at compile time. 
/// 
/// * This can cause problems on small stack devices, or large allocation volumes do to
/// stack crashing. This is where you over-allocate your stack causing the stack to 
/// overflow. This means that its up to the user to correctly setup the size, and
/// understand how that size is going to affect thier stack.  
pub struct HeaplessBits<const BYTES: usize> {
    /// ## `data`
    /// The byte array used to store the bits as flags.
    /// 
    /// ## Improvements
    /// We could allocate a buffer of `u64`, or even `usize` 
    /// to increase speed, however, due to rust's const
    /// eval system we can not do math at compile time. This
    /// causes problems with the user when they want to allocate
    /// only 4 bytes, and we want to have a 8 byte value.  
    data: HeaplessVec<u8, BYTES>
}

impl<const BYTES: usize> HeaplessBits<BYTES> {
    pub fn new() -> Self {
        Self {
            data: HeaplessVec::new()
        }
    }

    #[inline]
    const fn set_flag_in_byte(byte: u8, index: usize, flag: bool) -> u8 {
        (byte & (0xFF ^ (1 << index))) | ((if flag { 1 } else { 0 }) << index)
    }

    #[inline]
    const fn get_flag_from_byte(byte: u8, index: usize) -> bool {
        (byte & (1 << index)) > 0
    }

    #[inline]
    const fn get_offsets(index: usize) -> (usize, usize) {
        (index / 8, index % 8)
    }

    #[inline]
    fn get_byte(&self, index: usize) -> Result<u8, HeaplessVecErr> {
        if index >= BYTES {
            return Err(HeaplessVecErr::VectorFull);
        }

        if index > self.data.len() {
            return Ok(0);
        }

        Ok(*self.data.get(index)?)
    }

    #[inline]
    fn store_byte(&mut self, index: usize, byte: u8) -> Result<(), HeaplessVecErr> {
        while self.data.len() <= index {
            self.data.push_within_capacity(0)?;
        }

        *self.data.get_mut(index)? = byte;

        Ok(())
    }

    #[inline]
    pub fn set_bit(&mut self, index: usize, flag: bool) -> Result<(), HeaplessVecErr> {
        let (data_offset, bit_offset) = Self::get_offsets(index);
        let byte = self.get_byte(data_offset)?;

        self.store_byte(data_offset, Self::set_flag_in_byte(byte, bit_offset, flag))
    }

    #[inline]
    pub fn get_bit(&self, index: usize) -> Result<bool, HeaplessVecErr> {
        let (data_offset, bit_offset) = Self::get_offsets(index);
        Ok(Self::get_flag_from_byte(self.get_byte(data_offset)?, bit_offset))
    }

}

impl<const BYTES: usize> Default for HeaplessBits<BYTES> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::heapless_bits::HeaplessBits;
    use crate::heapless_vector::HeaplessVecErr;

    #[test]
    fn test_index_offsets() {
        assert_eq!(HeaplessBits::<0>::get_offsets(0), (0, 0));
        assert_eq!(HeaplessBits::<0>::get_offsets(1), (0, 1));
        assert_eq!(HeaplessBits::<0>::get_offsets(8), (1, 0));
        assert_eq!(HeaplessBits::<0>::get_offsets(9), (1, 1));
        assert_eq!(HeaplessBits::<0>::get_offsets(23), (2, 7));
    }

    #[test]
    fn test_get_flag_from_byte() {
        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1001010, 0), false);
        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1001010, 1), true);
        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1001010, 2), false);
        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1001010, 3), true);
        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1001010, 4), false);
        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1001010, 5), false);
        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1001010, 6), true);

        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1111111, 0), true);
        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1111111, 1), true);
        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1111111, 2), true);
        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1111111, 3), true);
        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1111111, 4), true);
        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1111111, 5), true);
        assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0b1111111, 6), true);

        for i in 0..8 {
            assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0x00, i), false);
            assert_eq!(HeaplessBits::<0>::get_flag_from_byte(0xFF, i), true);
        }
    }

    #[test]
    fn test_set_flag_in_byte() {
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b00000000, 0, true), 0b00000001);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b00000000, 1, true), 0b00000010);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b00000000, 2, true), 0b00000100);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b00000000, 3, true), 0b00001000);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b00000000, 4, true), 0b00010000);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b00000000, 5, true), 0b00100000);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b00000000, 6, true), 0b01000000);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b00000000, 7, true), 0b10000000);

        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b11111111, 0, false), 0b11111110);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b11111111, 1, false), 0b11111101);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b11111111, 2, false), 0b11111011);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b11111111, 3, false), 0b11110111);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b11111111, 4, false), 0b11101111);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b11111111, 5, false), 0b11011111);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b11111111, 6, false), 0b10111111);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b11111111, 7, false), 0b01111111);

        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b11100101, 0, false), 0b11100100);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b10101101, 6, false), 0b10101101);
        assert_eq!(HeaplessBits::<0>::set_flag_in_byte(0b11111111, 0, true), 0b11111111);
    }

    #[test]
    fn test_setting_bits_1_byte() {
        let mut bits = HeaplessBits::<1>::new();

        assert_eq!(bits.get_bit(10), Err(HeaplessVecErr::VectorFull));

        assert_eq!(bits.get_bit(0), Ok(false));
        assert_eq!(bits.get_bit(1), Ok(false));
        assert_eq!(bits.get_bit(2), Ok(false));
        assert_eq!(bits.get_bit(3), Ok(false));
        assert_eq!(bits.get_bit(4), Ok(false));
        assert_eq!(bits.get_bit(5), Ok(false));
        assert_eq!(bits.get_bit(6), Ok(false));
        assert_eq!(bits.get_bit(7), Ok(false));

        bits.set_bit(0, true).unwrap();

        assert_eq!(bits.get_bit(0), Ok(true));
        assert_eq!(bits.get_bit(1), Ok(false));
        assert_eq!(bits.get_bit(2), Ok(false));
        assert_eq!(bits.get_bit(3), Ok(false));
        assert_eq!(bits.get_bit(4), Ok(false));
        assert_eq!(bits.get_bit(5), Ok(false));
        assert_eq!(bits.get_bit(6), Ok(false));
        assert_eq!(bits.get_bit(7), Ok(false));

        bits.set_bit(1, true).unwrap();

        assert_eq!(bits.get_bit(0), Ok(true));
        assert_eq!(bits.get_bit(1), Ok(true));
        assert_eq!(bits.get_bit(2), Ok(false));
        assert_eq!(bits.get_bit(3), Ok(false));
        assert_eq!(bits.get_bit(4), Ok(false));
        assert_eq!(bits.get_bit(5), Ok(false));
        assert_eq!(bits.get_bit(6), Ok(false));
        assert_eq!(bits.get_bit(7), Ok(false));

        bits.set_bit(2, true).unwrap();

        assert_eq!(bits.get_bit(0), Ok(true));
        assert_eq!(bits.get_bit(1), Ok(true));
        assert_eq!(bits.get_bit(2), Ok(true));
        assert_eq!(bits.get_bit(3), Ok(false));
        assert_eq!(bits.get_bit(4), Ok(false));
        assert_eq!(bits.get_bit(5), Ok(false));
        assert_eq!(bits.get_bit(6), Ok(false));
        assert_eq!(bits.get_bit(7), Ok(false));

        bits.set_bit(3, true).unwrap();

        assert_eq!(bits.get_bit(0), Ok(true));
        assert_eq!(bits.get_bit(1), Ok(true));
        assert_eq!(bits.get_bit(2), Ok(true));
        assert_eq!(bits.get_bit(3), Ok(true));
        assert_eq!(bits.get_bit(4), Ok(false));
        assert_eq!(bits.get_bit(5), Ok(false));
        assert_eq!(bits.get_bit(6), Ok(false));
        assert_eq!(bits.get_bit(7), Ok(false));

        bits.set_bit(4, true).unwrap();

        assert_eq!(bits.get_bit(0), Ok(true));
        assert_eq!(bits.get_bit(1), Ok(true));
        assert_eq!(bits.get_bit(2), Ok(true));
        assert_eq!(bits.get_bit(3), Ok(true));
        assert_eq!(bits.get_bit(4), Ok(true));
        assert_eq!(bits.get_bit(5), Ok(false));
        assert_eq!(bits.get_bit(6), Ok(false));
        assert_eq!(bits.get_bit(7), Ok(false));

        bits.set_bit(5, true).unwrap();

        assert_eq!(bits.get_bit(0), Ok(true));
        assert_eq!(bits.get_bit(1), Ok(true));
        assert_eq!(bits.get_bit(2), Ok(true));
        assert_eq!(bits.get_bit(3), Ok(true));
        assert_eq!(bits.get_bit(4), Ok(true));
        assert_eq!(bits.get_bit(5), Ok(true));
        assert_eq!(bits.get_bit(6), Ok(false));
        assert_eq!(bits.get_bit(7), Ok(false));

        bits.set_bit(6, true).unwrap();

        assert_eq!(bits.get_bit(0), Ok(true));
        assert_eq!(bits.get_bit(1), Ok(true));
        assert_eq!(bits.get_bit(2), Ok(true));
        assert_eq!(bits.get_bit(3), Ok(true));
        assert_eq!(bits.get_bit(4), Ok(true));
        assert_eq!(bits.get_bit(5), Ok(true));
        assert_eq!(bits.get_bit(6), Ok(true));
        assert_eq!(bits.get_bit(7), Ok(false));

        bits.set_bit(7, true).unwrap();

        assert_eq!(bits.get_bit(0), Ok(true));
        assert_eq!(bits.get_bit(1), Ok(true));
        assert_eq!(bits.get_bit(2), Ok(true));
        assert_eq!(bits.get_bit(3), Ok(true));
        assert_eq!(bits.get_bit(4), Ok(true));
        assert_eq!(bits.get_bit(5), Ok(true));
        assert_eq!(bits.get_bit(6), Ok(true));
        assert_eq!(bits.get_bit(7), Ok(true));

        bits.set_bit(7, false).unwrap();

        assert_eq!(bits.get_bit(0), Ok(true));
        assert_eq!(bits.get_bit(1), Ok(true));
        assert_eq!(bits.get_bit(2), Ok(true));
        assert_eq!(bits.get_bit(3), Ok(true));
        assert_eq!(bits.get_bit(4), Ok(true));
        assert_eq!(bits.get_bit(5), Ok(true));
        assert_eq!(bits.get_bit(6), Ok(true));
        assert_eq!(bits.get_bit(7), Ok(false));

        bits.set_bit(0, false).unwrap();

        assert_eq!(bits.get_bit(0), Ok(false));
        assert_eq!(bits.get_bit(1), Ok(true));
        assert_eq!(bits.get_bit(2), Ok(true));
        assert_eq!(bits.get_bit(3), Ok(true));
        assert_eq!(bits.get_bit(4), Ok(true));
        assert_eq!(bits.get_bit(5), Ok(true));
        assert_eq!(bits.get_bit(6), Ok(true));
        assert_eq!(bits.get_bit(7), Ok(false));
    }
}
