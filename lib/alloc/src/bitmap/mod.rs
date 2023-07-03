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

use quantum_utils::bitset::BitSet;
use crate::bitmap::raw_bits::RawBitmap;
use crate::heap::GlobalAlloc;

mod raw_bits;


// FIXME: Rust is giving weird issues when using `Alloc = GlobalAlloc` in this type
pub struct Bitmap {
    raw: RawBitmap<GlobalAlloc>
}

impl Bitmap {
    const UNIT_SIZE_BYTES: usize = RawBitmap::<GlobalAlloc>::UNIT_SIZE;
    const UNIT_SIZE_BITS: usize = Self::UNIT_SIZE_BYTES * 8;

    pub const fn new() -> Self {
        Self {
            raw: RawBitmap::new()
        }
    }

    pub fn with_capacity(n_bits: usize) -> Self {
        let mut tmp = Self::new();
        tmp.raw.reserve(n_bits / Self::UNIT_SIZE_BITS);

        tmp
    }

    pub fn capacity(&self) -> usize {
        self.raw.cap * Self::UNIT_SIZE_BITS
    }

    fn capacity_deficit_in_allocations(&self, n_bit: usize) -> usize {
        if self.capacity() > n_bit {
            0
        } else {
            ((n_bit - self.capacity()) / Self::UNIT_SIZE_BITS) + 1
        }
    }

    pub fn set_bit(&mut self, n_bit: usize, flag: bool) {
        let capacity_diff = self.capacity_deficit_in_allocations(n_bit);
        if capacity_diff != 0 {
            self.raw.reserve(capacity_diff);
        }

        unsafe {
            let ptr = self.raw.ptr.as_ptr().add(n_bit / Self::UNIT_SIZE_BITS);
            *ptr = (*ptr).set_bit((n_bit % 8) as u8, flag);
        }
    }

    pub fn get_bit(&self, n_bit: usize) -> bool {
        let capacity_diff = self.capacity_deficit_in_allocations(n_bit);
        if capacity_diff != 0 {
            return false;
        }

        let return_flag;
        unsafe {
            let ptr = self.raw.ptr.as_ptr().add(n_bit / Self::UNIT_SIZE_BITS);
            return_flag = (*ptr).get_bit((n_bit % 8) as u8);
        }

        return_flag
    }
}

#[cfg(test)]
mod test {
    use crate::heap::{get_global_alloc, set_example_allocator};
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

        assert!(false, "{:#?}", get_global_alloc().allocations);
        assert_eq!(bitmap.get_bit(1000), false);
    }


}