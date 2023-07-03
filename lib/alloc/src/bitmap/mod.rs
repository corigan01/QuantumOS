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

use core::marker::PhantomData;
use quantum_utils::bitset::BitSet;
use crate::bitmap::raw_bits::RawBitmap;
use crate::heap::{AllocatorAPI, GlobalAlloc};

mod raw_bits;


// FIXME: Rust is giving weird issues when using `Alloc = GlobalAlloc` in this type
pub struct Bitmap<Alloc: AllocatorAPI = GlobalAlloc> {
    raw: RawBitmap<Alloc>,
    phn: PhantomData<Alloc>
}

impl Bitmap {
    const UNIT_SIZE_BYTES: usize = RawBitmap::<GlobalAlloc>::UNIT_SIZE_BYTES;
    const BITS_PER_UNIT: usize = RawBitmap::<GlobalAlloc>::BITS_PER_UNIT;

    pub const fn new() -> Self {
        Self {
            raw: RawBitmap::new(),
            phn: PhantomData
        }
    }

    pub fn with_capacity(n_bits: usize) -> Self {
        let mut tmp = Self::new();
        tmp.raw.reserve(n_bits / Self::BITS_PER_UNIT);

        tmp
    }

    pub fn capacity(&self) -> usize {
        self.raw.cap * Self::BITS_PER_UNIT
    }

    pub fn set_bit(&mut self, n_bit: usize, flag: bool) {
        let unit_index = n_bit / Self::BITS_PER_UNIT;
        let bit_index = n_bit % Self::BITS_PER_UNIT;

        let bit = self.raw.read(unit_index).set_bit(bit_index as u8, flag);
        self.raw.write(unit_index, bit);
    }

    pub fn get_bit(&self, n_bit: usize) -> bool {
        let unit_index = n_bit / Self::BITS_PER_UNIT;
        let bit_index = n_bit % Self::BITS_PER_UNIT;

        self.raw.read(unit_index).get_bit(bit_index as u8)
    }
}

#[cfg(test)]
mod test {
    use crate::heap::set_example_allocator;
    use super::*;

    #[test]
    fn test_set_one_bit() {
        set_example_allocator(4096);
        let mut bitmap = Bitmap::new();

        bitmap.set_bit(0, true);

        assert_eq!(bitmap.get_bit(0), true);
    }

    #[test]
    fn test_some_bits() {
        set_example_allocator(4096);
        let mut bitmap = Bitmap::new();

        bitmap.set_bit(0, true);
        bitmap.set_bit(1, true);
        bitmap.set_bit(2, false);
        bitmap.set_bit(3, false);
        bitmap.set_bit(4, true);

        assert_eq!(bitmap.get_bit(0), true);
        assert_eq!(bitmap.get_bit(1), true);
        assert_eq!(bitmap.get_bit(2), false);
        assert_eq!(bitmap.get_bit(3), false);
        assert_eq!(bitmap.get_bit(4), true);
        assert_eq!(bitmap.get_bit(5), false);

    }

    #[test]
    fn test_getting_bits_across_bytes() {
        set_example_allocator(4096);
        let mut bitmap = Bitmap::new();

        bitmap.set_bit(10, true);
        bitmap.set_bit(112, true);
        bitmap.set_bit(150, false);
        bitmap.set_bit(132, false);
        bitmap.set_bit(14, true);

        assert_eq!(bitmap.get_bit(10), true);
        assert_eq!(bitmap.get_bit(112), true);
        assert_eq!(bitmap.get_bit(150), false);
        assert_eq!(bitmap.get_bit(132), false);
        assert_eq!(bitmap.get_bit(14), true);
        assert_eq!(bitmap.get_bit(200), false);

    }

    #[test]
    fn test_setting_many_bits() {
        set_example_allocator(4096);
        let mut bitmap = Bitmap::with_capacity(1000);

        for i in 0..1000 {
            bitmap.set_bit(i, true);
        }

        for i in 0..1000 {
            assert_eq!(bitmap.get_bit(i), true);
        }

        assert_eq!(bitmap.get_bit(1000), false);
    }


}