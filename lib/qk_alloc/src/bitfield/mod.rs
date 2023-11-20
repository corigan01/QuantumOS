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

use crate::bitfield::raw_bits::RawBitmap;
use crate::heap::{AllocatorAPI, GlobalAlloc};
use core::marker::PhantomData;
use quantum_utils::bitset::BitSet;

mod raw_bits;

pub struct Bitmap<Alloc: AllocatorAPI = GlobalAlloc> {
    raw: RawBitmap<Alloc>,
    highest_set: usize,
    phn: PhantomData<Alloc>,
}

impl Bitmap {
    const BITS_PER_UNIT: usize = RawBitmap::<GlobalAlloc>::BITS_PER_UNIT;

    pub const fn new() -> Self {
        Self {
            raw: RawBitmap::new(),
            highest_set: 0,
            phn: PhantomData,
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

    pub fn highest_set(&self) -> usize {
        self.highest_set
    }

    pub fn set_bit(&mut self, n_bit: usize, flag: bool) {
        if n_bit > self.highest_set {
            self.highest_set = n_bit;
        }

        let unit_index = n_bit / Self::BITS_PER_UNIT;
        let bit_index = n_bit % Self::BITS_PER_UNIT;

        // We don't need to allocate if its not setting a bit,
        // only allocate if we *must*!
        if !flag && unit_index > self.capacity() {
            return;
        }

        let bit = self.raw.read(unit_index).set_bit(bit_index as u8, flag);
        self.raw.write(unit_index, bit);
    }

    pub fn get_bit(&self, n_bit: usize) -> bool {
        let unit_index = n_bit / Self::BITS_PER_UNIT;
        let bit_index = n_bit % Self::BITS_PER_UNIT;

        self.raw.read(unit_index).get_bit(bit_index as u8)
    }

    pub fn set_many(&mut self, index: usize, flag: bool, qty: usize) {
        for i in index..(index + qty) {
            self.set_bit(i, flag);
        }
    }

    pub fn first_of(&self, flag: bool) -> Option<usize> {
        let skip_if_equal = if flag { 0 } else { usize::MAX };

        if !flag && self.raw.cap == 0 {
            return Some(0);
        }

        for unit_index in 0..self.raw.cap {
            let unit_value = self.raw.read(unit_index);
            if unit_value == skip_if_equal {
                continue;
            }

            for bit_index in 0..Self::BITS_PER_UNIT {
                let bit_flag = unit_value.get_bit(bit_index as u8);

                if bit_flag != flag {
                    continue;
                }

                return Some(unit_index * Self::BITS_PER_UNIT + bit_index);
            }
        }

        None
    }

    pub fn first_of_many(&self, flag: bool, qty: usize) -> Option<usize> {
        assert_ne!(qty, 0, "Qty must not be zero when finding a bit");

        let skip_if_equal = if flag { 0 } else { usize::MAX };

        if !flag && self.raw.cap == 0 {
            return Some(0);
        }

        let mut last_found: Option<usize> = None;
        let mut qty_found = 0;

        for unit_index in 0..self.raw.cap {
            let unit_value = self.raw.read(unit_index);
            if unit_value == skip_if_equal {
                last_found = None;
                qty_found = 0;
                continue;
            }

            for bit_index in 0..Self::BITS_PER_UNIT {
                let bit_flag = unit_value.get_bit(bit_index as u8);

                if bit_flag != flag {
                    last_found = None;
                    qty_found = 0;
                    continue;
                }

                if last_found.is_none() {
                    last_found = Some(unit_index * Self::BITS_PER_UNIT + bit_index);
                }

                qty_found += 1;

                if qty_found == qty {
                    return last_found;
                }
            }
        }

        if qty_found == qty {
            return last_found;
        }

        None
    }

    pub fn reset_to_zero(&mut self) {
        self.raw.reset();
        self.highest_set = 0;
    }

    pub fn iter(&self) -> BitmapIter {
        BitmapIter::from_bitmap_ref(&self)
    }
}

pub struct BitmapIter<'a> {
    bitmap: &'a Bitmap,
    bit_index: usize,
}

impl<'a> BitmapIter<'a> {
    fn from_bitmap_ref(bitmap: &'a Bitmap) -> Self {
        Self {
            bitmap,
            bit_index: 0,
        }
    }
}

impl<'a> Iterator for BitmapIter<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bit_index > self.bitmap.highest_set {
            None
        } else {
            let flag = self.bitmap.get_bit(self.bit_index);
            self.bit_index += 1;

            Some(flag)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::heap::set_example_allocator;

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

    #[test]
    fn test_bitmap_iterator() {
        set_example_allocator(4096);
        let mut bitmap = Bitmap::new();

        for i in 0..100 {
            bitmap.set_bit(i, true);
        }

        for (index, bit) in bitmap.iter().enumerate() {
            assert_eq!(bit, true, "Was not true on {}", index);
        }
    }

    #[test]
    fn test_first_of_bitmap() {
        set_example_allocator(4096);
        let mut bitmap = Bitmap::new();

        bitmap.set_bit(10, true);
        assert_eq!(bitmap.first_of(true), Some(10));
        assert_eq!(bitmap.first_of(false), Some(0));

        bitmap.set_bit(0, true);
        assert_eq!(bitmap.first_of(true), Some(0));
        assert_eq!(bitmap.first_of(false), Some(1));
    }

    #[test]
    fn test_zero_sized_bitmap() {
        let bitmap = Bitmap::new();

        assert_eq!(bitmap.first_of(false), Some(0));
        assert_eq!(bitmap.first_of(true), None);
    }

    #[test]
    fn test_first_of_many() {
        set_example_allocator(4096);
        let mut bitmap = Bitmap::new();

        for i in 0..1000 {
            bitmap.set_bit(i, true);
        }

        assert_eq!(bitmap.first_of(false), Some(1000));
        assert_eq!(bitmap.first_of_many(true, 10), Some(0));
        assert_eq!(bitmap.first_of_many(true, 1000), Some(0));
        assert_eq!(bitmap.first_of_many(false, 10), Some(1000));

        bitmap.set_bit(0, false);

        assert_eq!(bitmap.first_of(false), Some(0));
        assert_eq!(bitmap.first_of_many(false, 2), Some(1000));
    }
}
