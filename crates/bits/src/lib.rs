#![no_std]
use core::ops::RangeBounds;

/// # Bit Manipulation
/// A Simple trait to help with setting and un-setting bits in types.
pub trait BitManipulation: Sized {
    /// # Set Bit
    /// Set a single bit in the given type.
    fn set_bit<B>(&mut self, bit: B, set: bool) -> &mut Self
    where
        B: Into<u8>;

    /// # Set Bit Range
    /// Set a range of bits in the given type.
    fn set_bit_range<R, B>(&mut self, bit: R, set: B) -> &mut Self
    where
        R: RangeBounds<Self>,
        B: Into<Self>;

    /// # Get Bit
    /// Get a single bit in the given type.
    fn get_bit<B>(&self, bit: B) -> bool
    where
        B: Into<u8>;

    /// # Get Bit Range
    /// Get a range of bits in the given type.
    fn get_bit_range<R>(&self, bit: R) -> Self
    where
        R: RangeBounds<Self>;
}

/// # Bit Manipulation Impl
/// Implement this trait for many types.
/// FIXME: We should use something like PrimInt from the num-traits create
///        to provide a `impl<T: PrimInt> BitManipulation for T {}`.
macro_rules! bit_manipulation_impl {
    ($($t:ty)*) => ($(
     impl BitManipulation for $t {
        /// # Set Bit
        /// Set a single bit in the given type.
        fn set_bit<B>(&mut self, bit: B, set: bool) -> &mut Self
        where
            B: Into<u8>,
        {
            let bit: u8 = bit.into();
            let self_bits = (core::mem::size_of::<Self>() * 8) as u8;

            debug_assert!(
                bit <= self_bits,
                "Bit '{bit}' is larger then type's total bits of '{self_bits}'!"
            );

            if set {
                *self |= 1 << bit;
            } else {
                *self &= !(1 << bit);
            }

            self
        }

        /// # Get Bit
        /// Get a single bit in the given type.
        fn get_bit<B>(&self, bit: B) -> bool
        where
            B: Into<u8> {
            let bit: u8 = bit.into();
            let self_bits = (core::mem::size_of::<Self>() * 8) as u8;

            debug_assert!(
                bit <= self_bits,
                "Bit '{bit}' is larger then type's total bits of '{self_bits}'!"
            );

            *self & (1 << bit) != 0
        }

        /// # Get Bit Range
        /// Get a range of bits in the given type.
        fn get_bit_range<R>(&self, bit: R) -> Self
        where
            R: RangeBounds<Self>,
        {
            let self_bits = (core::mem::size_of::<Self>() * 8) as Self;
            let true_bit_start = match bit.start_bound() {
                core::ops::Bound::Included(&value) => value,
                core::ops::Bound::Excluded(&value) => value + 1,
                core::ops::Bound::Unbounded => 0 as Self,
            };

            let true_bit_end = match bit.end_bound() {
                core::ops::Bound::Included(&value) => value + 1,
                core::ops::Bound::Excluded(&value) => value,
                core::ops::Bound::Unbounded => self_bits,
            };

            debug_assert!(
                true_bit_start <= self_bits,
                "Bit Start '{true_bit_start}' is larger then type's total bits of '{self_bits}'!"
            );

            debug_assert!(
                true_bit_end <= self_bits,
                "Bit End '{true_bit_end}' is larger then type's total bits of '{self_bits}'!"
            );

            debug_assert!(
                true_bit_end >= true_bit_start,
                "Bit Start '{true_bit_start}' must be less then Bit End '{true_bit_end}'!"
            );

            let bits = *self << (self_bits - true_bit_end) >> (self_bits - true_bit_end);

            bits >> true_bit_start
        }

        /// # Set Bit Range
        /// Set a range of bits in the given type.
        fn set_bit_range<R, B>(&mut self, bit: R, set: B) -> &mut Self
        where
            R: RangeBounds<Self>,
            B: Into<Self>,
        {
            let set_bits: Self = set.into();
            let self_bits = (core::mem::size_of::<Self>() * 8) as Self;
            let true_bit_start = match bit.start_bound() {
                core::ops::Bound::Included(&value) => value,
                core::ops::Bound::Excluded(&value) => value + 1,
                core::ops::Bound::Unbounded => 0 as Self,
            };

            let true_bit_end = match bit.end_bound() {
                core::ops::Bound::Included(&value) => value,
                core::ops::Bound::Excluded(&value) => value - 1,
                core::ops::Bound::Unbounded => self_bits,
            };

            let true_bit_diff = true_bit_end - true_bit_start;

            debug_assert!(
                true_bit_start <= self_bits,
                "Bit Start '{true_bit_start}' is larger then type's total bits of '{self_bits}'!"
            );

            debug_assert!(
                true_bit_end <= self_bits,
                "Bit End '{true_bit_end}' is larger then type's total bits of '{self_bits}'!"
            );

            debug_assert!(
                true_bit_end >= true_bit_start,
                "Bit Start '{true_bit_start}' must be less then Bit End '{true_bit_end}'!"
            );

            debug_assert!(
                set_bits >> true_bit_diff <= 1,
                "The setting bits '0b{set_bits:0b}' cannot be more bits then amount specified {true_bit_diff}!"
            );

            // Generate Mask bits
            // TODO: There must be a better way of doing this?
            let mut mask = 1;
            for _ in 0..true_bit_diff {
                mask <<= 1;
                mask |= 1;
            }
            mask <<= true_bit_start;

            // Combine everything together
            *self = (*self & !mask) | (set_bits << true_bit_start);
            self
        }
    }
    )*)
}

bit_manipulation_impl! { u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 usize isize }

#[cfg(test)]
mod test {
    use super::*;

    const TEST_EXAMPLES_U8: [(u8, u8, bool, u8); 10] = [
        // (Value, Bit, Flag, Expected)
        (0b00000000, 2, true, 0b00000100),
        (0b00000000, 2, false, 0b00000000),
        (0b11111111, 0, false, 0b11111110),
        (0b10101010, 0, true, 0b10101011),
        (0b11110000, 7, false, 0b01110000),
        (0b01111111, 7, true, 0b11111111),
        (0b10101000, 4, true, 0b10111000),
        (0b00000000, 7, false, 0b00000000),
        (0b11111111, 4, true, 0b11111111),
        (0b11001100, 0, true, 0b11001101),
    ];

    #[test]
    fn test_setting_bits() {
        for (test, bit, flag, expected) in TEST_EXAMPLES_U8 {
            let mut value = test;
            value.set_bit(bit, flag);
            assert_eq!(
                value, expected,
                "Value does not equal expected, 0b{value:08b} != 0b{expected:08b}"
            );
        }
    }

    #[test]
    fn test_getting_bits() {
        // Here we know that the `Expected` at `Bit` should always have `Flag` enabled
        for (_, bit, flag, expected) in TEST_EXAMPLES_U8 {
            let value = expected.get_bit(bit);
            assert_eq!(value, flag, "Failed getting bits, does not equal expected.");
        }
    }

    #[test]
    fn test_setting_bit_range_one_bit() {
        for (test, bit, flag, expected) in TEST_EXAMPLES_U8 {
            let mut value = test;
            let flag_value = if flag { 1 } else { 0 };
            value.set_bit_range(bit..=bit, flag_value);
            assert_eq!(
                value, expected,
                "Value does not equal expected, 0b{value:08b} != 0b{expected:08b}"
            );
        }
    }

    #[test]
    fn test_getting_bit_range_one_bit() {
        for (_, bit, flag, expected) in TEST_EXAMPLES_U8 {
            let flag_value = if flag { 1 } else { 0 };
            let value = expected.get_bit_range(bit..=bit);
            assert_eq!(
                value, flag_value,
                "Failed getting bits, does not equal expected."
            );
        }
    }

    #[test]
    fn test_setting_bit_range() {
        assert_eq!(*0b00000000_u8.set_bit_range(0..=7, 0), 0b00000000_u8);
        assert_eq!(*0b11111111_u8.set_bit_range(0..=7, 0), 0b00000000_u8);
        assert_eq!(*0b01010101_u8.set_bit_range(0..=7, 0), 0b00000000_u8);
        assert_eq!(*0b00000000_u8.set_bit_range(0..=7, 0xFF), 0b11111111_u8);
    }

    #[test]
    fn test_set_bit_same_as_shift_ore() {
        assert_eq!(*0b00000u8.set_bit(0, true), 1 << 0);
        assert_eq!(*0b00000u8.set_bit(1, true), 1 << 1);
        assert_eq!(*0b00000u8.set_bit(2, true), 1 << 2);
        assert_eq!(*0b00000u8.set_bit(3, true), 1 << 3);
        assert_eq!(*0b00000u8.set_bit(4, true), 1 << 4);
    }
}
