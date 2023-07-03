/*!
```text
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
```

# Bitset
A basic QuantumOS helper tool to make setting bits in numbers much easier!

## Safety
Bitset is completely safe as it does not need to allocate memory, and has safe guards that will
panic upon overflow before operations happen. This allows the user to trust that when they set a bit,
the operation will happen correctly as performed.

## Usage
A basic rust example is as follows for setting bit 2 to true
```rust
use quantum_utils::bitset::BitSet;

let output = 0_u8.set_bit(2, true);
```

Bitset can also be used to set a range of bits!
```rust
use quantum_utils::bitset::BitSet;

let output = 255_u8.set_bits(0..8, 0);
```
*/

use core::mem::size_of;
use core::ops::Range;

pub struct BitsIter<Type: Sized> {
    value: Type,
    forward_index: usize,
    back_index: usize
}

impl<Type> Iterator for BitsIter<Type>
    where Type: BitSet {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.forward_index >= Type::max_bits()
            || ((Type::max_bits() - self.back_index) + self.forward_index) >= Type::max_bits() {
            return None;
        }

        self.forward_index += 1;
        Some(self.value.get_bit((self.forward_index - 1) as u8 ))
    }
}

impl<Type> DoubleEndedIterator for BitsIter<Type>
    where Type: BitSet {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back_index == 0
            || ((Type::max_bits() - self.back_index) + self.forward_index) >= Type::max_bits() {
            return None;
        }

        let return_value = Some(self.value.get_bit((self.back_index - 1) as u8));
        self.back_index -= 1;

        return_value
    }
}

/// # Bitset trait
/// The main traits for settings bits!
pub trait BitSet {
    fn set_bit(&mut self, bit: u8, v: bool) -> Self;
    fn set_bits(&mut self, r: Range<u8>, bits: u64) -> Self;

    fn get_bit(&self, bit: u8) -> bool;
    fn get_bits(&self, r: Range<u8>) -> Self;

    fn bits(&self) -> BitsIter<Self> where Self: Sized;

    fn max_bits() -> usize;
}

macro_rules! bitset_impl {
    ($($t:ty)*) => ($(
        impl BitSet for $t {

            /// # set_bit
            ///
            /// `set_bit` can toggle a bit in any stream of numbers. This can include setting bit `1` to
            /// `true`, or setting bit `7` to `false`.
            ///
            /// # Safety
            /// `set_bit` has protections for when you try to set a bit outside the range of the type you are
            /// setting. For example, if you try to set bit `14` in a `u8` type, this will panic as bit `14`
            /// does not exist in a u8 type.
            ///
            /// # Portability
            /// The `set_bit` trait can be implemented for any type as long as the type contains a stream
            /// of numbers ( without gaps ) that the user would like to toggle. This could be a custom struct
            /// that contains a primitive type.
            ///
            /// # Use
            /// ```rust
            /// use quantum_utils::bitset::BitSet;
            ///
            /// let value = 16_u8.set_bit(0, true);
            /// assert_eq!(value, 17_u8);
            ///
            /// ```
            fn set_bit(&mut self, bit: u8, v: bool) -> Self {
                use core::mem::size_of;

                let self_size_bits = size_of::<Self>() * 8;
                if bit > self_size_bits as u8 { panic!("Tried to set a bit outside the range of the current type!"); }

                if v {
                    *self |= 1 << bit;
                }
                else {
                    *self &= !(1 << bit);
                }

                *self
            }

            /// # set_bits
            /// `set_bits` can toggle multiple bits in any stream of numbers. This can include setting
            /// bits `1 - 3` to `true`, or setting bits `2-7` to `false`.
            ///
            /// # Safety
            /// `set_bits` has protections for when you try to set a bit outside the range of the type
            /// you are setting. For example, if you try to set bit `14` in a `u8` type, this will panic as
            /// bit `14` does not exist in a u8 type.
            ///
            /// # Portability
            /// The `set_bits` trait can be implemented for any type as long as the type contains a stream
            /// of numbers ( without gaps ) that the user would like to toggle. This could be a custom struct
            /// that contains a primitive type.
            ///
            /// # Use
            /// ```rust
            /// use quantum_utils::bitset::BitSet;
            ///
            /// let value = 16_u8.set_bits(0..2, 0b01);
            /// assert_eq!(value, 17_u8);
            ///
            /// ```
            fn set_bits(&mut self, r: Range<u8>, bits: u64) -> Self {
                let bit_out_of_range = bits as u64 / (2 as u64).pow(r.end as u32);
                if bit_out_of_range > 0 { panic!("Tried to set a bit outside the range provided!"); }

                let mut dev = 0;
                for bit in r {
                    self.set_bit(bit, ((bits >> dev) & 0b1) == 1);
                    dev += 1;
                }

                *self
            }

            /// # get_bit
            /// `get_bit` can return a bit in any stream of numbers. This can include getting any bits within
            /// the type limits of the value.
            ///
            /// # Safety
            /// `get_bit` has protections for when you try to get a bit outside the range of the type you are
            /// using. For example, if you try to get bit `14` in a `u8` type, this will panic as bit `14`
            /// does not exist in a u8 type.
            ///
            /// # Portability
            /// The `get_bit` trait can be implemented for any type as long as the type contains a stream
            /// of numbers ( without gaps ) that the user would like to return. This could be a custom struct
            /// that contains a primitive type.
            ///
            /// # Use
            /// ```rust
            /// use quantum_utils::bitset::BitSet;
            ///
            /// let value = 16_u8.get_bit(0);
            /// assert_eq!(value, false);
            ///
            /// ```
            fn get_bit(&self, bit: u8) -> bool {
                use core::mem::size_of;

                let self_size_bits = size_of::<Self>() * 8;
                if bit >= self_size_bits as u8 { panic!("Tried to get a bit outside the range of the current type!"); }

                (self >> bit) & 0b1 == 0b1
            }

            /// # get_bits
            /// `get_bit` can return a bit in any stream of numbers. This can include getting any bits within
            /// the type limits of the value.
            ///
            /// # Safety
            /// `get_bit` has protections for when you try to get a bit outside the range of the type you are
            /// using. For example, if you try to get bit `14` in a `u8` type, this will panic as bit `14`
            /// does not exist in a u8 type.
            ///
            /// # Portability
            /// The `get_bit` trait can be implemented for any type as long as the type contains a stream
            /// of numbers ( without gaps ) that the user would like to return. This could be a custom struct
            /// that contains a primitive type.
            ///
            /// # Use
            /// ```rust
            /// use quantum_utils::bitset::BitSet;
            ///
            /// let value = 4_u8.get_bits(0..2);
            /// assert_eq!(value, 0_u8);
            ///
            /// ```
            fn get_bits(&self, r: Range<u8>) -> Self {
                use core::mem::size_of;

                let bits = *self << (((size_of::<Self>() * 8) as Self) - (r.end as Self))
                >> (((size_of::<Self>() * 8) as Self) - (r.end as Self));

                bits >> r.start
            }

            fn bits(&self) -> BitsIter<Self> {
                BitsIter {
                    value: *self,
                    forward_index: 0,
                    back_index: Self::max_bits()
                }
            }

            fn max_bits() -> usize {
                size_of::<Self>() * 8
            }
        }
    )*)
}

bitset_impl! {  u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 usize isize  }

#[cfg(test)]
mod tests {
    use crate::bitset::BitSet;

    #[test]
    fn set_bit_test() {
        assert_eq!(0_u8.set_bit(7, true), 128_u8);
        assert_eq!(123_u16.set_bit(5, false), 91_u16);
        assert_eq!(141742579807_u64.set_bit(38, true), 416620486751_u64);
    }

    #[test]
    fn set_bits_test() {
        assert_eq!(0_u8.set_bits(3..7, 0b101), 40_u8);
        assert_eq!(255_u16.set_bits(0..8, 0), 0_u16);
        assert_eq!(0_u8.set_bits(0..8, 255), 255_u8);
    }

    #[test]
    fn get_bit_test() {
        assert_eq!(138.get_bit(3), true);
        assert_eq!(412.get_bit(6), false);
        assert_eq!(141742579807_u64.get_bit(37), true);
    }

    #[test]
    fn get_bits_test() {
        assert_eq!(321_u32.get_bits(0..9), 321_u32);
        assert_eq!(42_u32.get_bits(6..8), 0_u32);
        assert_eq!(4921949_u64.get_bits(16..20), 0b1011_u64);
    }

    #[test]
    fn bits_iterator_test() {
        let value: u8 = 0b10011010;
        let mut bits = value.bits();

        assert_eq!(
            (
                bits.next(),
                bits.next(),
                bits.next(),
                bits.next(),
                bits.next(),
                bits.next(),
                bits.next(),
                bits.next(),
                bits.next()
            ),
            (
                Some(false),
                Some(true),
                Some(false),
                Some(true),
                Some(true),
                Some(false),
                Some(false),
                Some(true),
                None
            )
        );
    }

    #[test]
    fn bits_iterator_test_rev() {
        let value: u8 = 0b10011010;
        let mut bits = value.bits().rev();

        assert_eq!(
            (
                bits.next(),
                bits.next(),
                bits.next(),
                bits.next(),
                bits.next(),
                bits.next(),
                bits.next(),
                bits.next(),
                bits.next()
            ),
            (
                Some(true),
                Some(false),
                Some(false),
                Some(true),
                Some(true),
                Some(false),
                Some(true),
                Some(false),
                None
            )
        );
    }
}
