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
use quantum_os::bitset::BitSet;

let output = 0_u8.set_bit(2, true);
```

Bitset can also be used to set a range of bits!
```rust
use quantum_os::bitset::BitSet;

let output = 255_u8.set_bit_range(0..8, 0);
```
*/

use core::ops::Range;

/// # Bitset trait
/// The main traits for settings bits!
pub trait BitSet {
    fn set_bit(&mut self, bit: u8, v: bool) -> Self;
    fn set_bit_range(&mut self, r: Range<u8>, bits: u64) -> Self;

    fn get_bit(&self, bit: u8) -> Self;
}

/// # Bitset for u8
impl BitSet for u8 {

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
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.set_bit(0, true);
    /// assert_eq!(value, 17);
    ///
    /// ```
    fn set_bit(&mut self, bit: u8, v: bool) -> Self {
        use core::mem::size_of;

        let self_size_bits = size_of::<Self>() * 8;
        if bit > self_size_bits as u8 { panic!("Tried to set a bit outside the range of the current type!"); }

        let calculate_bit = (2 as Self).pow(bit as u32);
        *self = if v { *self | calculate_bit } else { *self & ( calculate_bit ^ Self::MAX) };

        *self
    }

    /// # set_bit_range
    /// `set_bit_range` can toggle multiple bits in any stream of numbers. This can include setting
    /// bits `1 - 3` to `true`, or setting bits `2-7` to `false`.
    ///
    /// # Safety
    /// `set_bit_range` has protections for when you try to set a bit outside the range of the type
    /// you are setting. For example, if you try to set bit `14` in a `u8` type, this will panic as
    /// bit `14` does not exist in a u8 type.
    ///
    /// # Portability
    /// The `set_bit_range` trait can be implemented for any type as long as the type contains a stream
    /// of numbers ( without gaps ) that the user would like to toggle. This could be a custom struct
    /// that contains a primitive type.
    ///
    /// # Use
    /// ```rust
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.set_bit_range(0..2, 0b01);
    /// assert_eq!(value, 17);
    ///
    /// ```
    fn set_bit_range(&mut self, r: Range<u8>, bits: u64) -> Self {
        let bit_out_of_range = bits as u64 / (2 as u64).pow(r.end as u32);
        if bit_out_of_range > 0 { panic!("Tried to set a bit outside the range provided!"); }

        let mut dev = 0;
        for bit in r {
            self.set_bit(bit, (bits >> dev & 1) == 1);

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
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.get_bit(0);
    /// assert_eq!(value, 0);
    ///
    /// ```
    fn get_bit(&self, bit: u8) -> Self {
        use core::mem::size_of;

        let self_size_bits = size_of::<Self>() * 8;
        if bit > self_size_bits as u8 { panic!("Tried to get a bit outside the range of the current type!"); }

        (self >> bit) & 0b1
    }
}

/// # Bitset for u16
impl BitSet for u16 {

    /// # set_bit
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
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.set_bit(0, true);
    /// assert_eq!(value, 17);
    ///
    /// ```
    fn set_bit(&mut self, bit: u8, v: bool) -> Self {
        use core::mem::size_of;

        let self_size_bits = size_of::<Self>() * 8;
        if bit > self_size_bits as u8 { panic!("Tried to set a bit outside the range of the current type!"); }

        let calculate_bit = (2 as Self).pow(bit as u32);
        *self = if v { *self | calculate_bit } else { *self & ( calculate_bit ^ Self::MAX) };

        *self
    }

    /// # set_bit_range
    /// `set_bit_range` can toggle multiple bits in any stream of numbers. This can include setting
    /// bits `1 - 3` to `true`, or setting bits `2-7` to `false`.
    ///
    /// # Safety
    /// `set_bit_range` has protections for when you try to set a bit outside the range of the type
    /// you are setting. For example, if you try to set bit `14` in a `u8` type, this will panic as
    /// bit `14` does not exist in a u8 type.
    ///
    /// # Portability
    /// The `set_bit_range` trait can be implemented for any type as long as the type contains a stream
    /// of numbers ( without gaps ) that the user would like to toggle. This could be a custom struct
    /// that contains a primitive type.
    ///
    /// # Use
    /// ```rust
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.set_bit_range(0..2, 0b01);
    /// assert_eq!(value, 17);
    ///
    /// ```
    fn set_bit_range(&mut self, r: Range<u8>, bits: u64) -> Self {
        let bit_out_of_range = bits as u64 / (2 as u64).pow(r.end as u32);
        if bit_out_of_range > 0 { panic!("Tried to set a bit outside the range provided!"); }

        let mut dev = 0;
        for bit in r {
            self.set_bit(bit, (bits >> dev & 1) == 1);

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
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.get_bit(0);
    /// assert_eq!(value, 0);
    ///
    /// ```
    fn get_bit(&self, bit: u8) -> Self {
        use core::mem::size_of;

        let self_size_bits = size_of::<Self>() * 8;
        if bit > self_size_bits as u8 { panic!("Tried to get a bit outside the range of the current type!"); }

        (self >> bit) & 0b1
    }
}

/// # Bitset for u32
impl BitSet for u32 {

    /// # set_bit
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
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.set_bit(0, true);
    /// assert_eq!(value, 17);
    ///
    /// ```
    fn set_bit(&mut self, bit: u8, v: bool) -> Self {
        use core::mem::size_of;

        let self_size_bits = size_of::<Self>() * 8;
        if bit > self_size_bits as u8 { panic!("Tried to set a bit outside the range of the current type!"); }

        let calculate_bit = (2 as Self).pow(bit as u32);
        *self = if v { *self | calculate_bit } else { *self & ( calculate_bit ^ Self::MAX) };

        *self
    }

    /// # set_bit_range
    /// `set_bit_range` can toggle multiple bits in any stream of numbers. This can include setting
    /// bits `1 - 3` to `true`, or setting bits `2-7` to `false`.
    ///
    /// # Safety
    /// `set_bit_range` has protections for when you try to set a bit outside the range of the type
    /// you are setting. For example, if you try to set bit `14` in a `u8` type, this will panic as
    /// bit `14` does not exist in a u8 type.
    ///
    /// # Portability
    /// The `set_bit_range` trait can be implemented for any type as long as the type contains a stream
    /// of numbers ( without gaps ) that the user would like to toggle. This could be a custom struct
    /// that contains a primitive type.
    ///
    /// # Use
    /// ```rust
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.set_bit_range(0..2, 0b01);
    /// assert_eq!(value, 17);
    ///
    /// ```
    fn set_bit_range(&mut self, r: Range<u8>, bits: u64) -> Self {
        let bit_out_of_range = bits as u64 / (2 as u64).pow(r.end as u32);
        if bit_out_of_range > 0 { panic!("Tried to set a bit outside the range provided!"); }

        let mut dev = 0;
        for bit in r {
            self.set_bit(bit, (bits >> dev & 1) == 1);

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
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.get_bit(0);
    /// assert_eq!(value, 0);
    ///
    /// ```
    fn get_bit(&self, bit: u8) -> Self {
        use core::mem::size_of;

        let self_size_bits = size_of::<Self>() * 8;
        if bit > self_size_bits as u8 { panic!("Tried to get a bit outside the range of the current type!"); }

        (self >> bit) & 0b1
    }
}

/// # Bitset for u64
impl BitSet for u64 {

    /// # set_bit
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
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.set_bit(0, true);
    /// assert_eq!(value, 17);
    ///
    /// ```
    fn set_bit(&mut self, bit: u8, v: bool) -> Self {
        use core::mem::size_of;

        let self_size_bits = size_of::<Self>() * 8;
        if bit > self_size_bits as u8 { panic!("Tried to set a bit outside the range of the current type!"); }

        let calculate_bit = (2 as Self).pow(bit as u32);
        *self = if v { *self | calculate_bit } else { *self & ( calculate_bit ^ Self::MAX) };

        *self
    }

    /// # set_bit_range
    /// `set_bit_range` can toggle multiple bits in any stream of numbers. This can include setting
    /// bits `1 - 3` to `true`, or setting bits `2-7` to `false`.
    ///
    /// # Safety
    /// `set_bit_range` has protections for when you try to set a bit outside the range of the type
    /// you are setting. For example, if you try to set bit `14` in a `u8` type, this will panic as
    /// bit `14` does not exist in a u8 type.
    ///
    /// # Portability
    /// The `set_bit_range` trait can be implemented for any type as long as the type contains a stream
    /// of numbers ( without gaps ) that the user would like to toggle. This could be a custom struct
    /// that contains a primitive type.
    ///
    /// # Use
    /// ```rust
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.set_bit_range(0..2, 0b01);
    /// assert_eq!(value, 17);
    ///
    /// ```
    fn set_bit_range(&mut self, r: Range<u8>, bits: u64) -> Self {
        let bit_out_of_range = bits as u64 / (2 as u64).pow(r.end as u32);
        if bit_out_of_range > 0 { panic!("Tried to set a bit outside the range provided!"); }

        let mut dev = 0;
        for bit in r {
            self.set_bit(bit, (bits >> dev & 1) == 1);

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
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.get_bit(0);
    /// assert_eq!(value, 0);
    ///
    /// ```
    fn get_bit(&self, bit: u8) -> Self {
        use core::mem::size_of;

        let self_size_bits = size_of::<Self>() * 8;
        if bit > self_size_bits as u8 { panic!("Tried to get a bit outside the range of the current type!"); }

        (self >> bit) & 0b1
    }
}

/// # Bitset for i32
impl BitSet for i32 {

    /// # set_bit
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
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.set_bit(0, true);
    /// assert_eq!(value, 17);
    ///
    /// ```
    fn set_bit(&mut self, bit: u8, v: bool) -> Self {
        use core::mem::size_of;

        let self_size_bits = size_of::<Self>() * 8;
        if bit > self_size_bits as u8 { panic!("Tried to set a bit outside the range of the current type!"); }

        let calculate_bit = (2 as Self).pow(bit as u32);
        *self = if v { *self | calculate_bit } else { *self & ( calculate_bit ^ Self::MAX) };

        *self
    }

    /// # set_bit_range
    /// `set_bit_range` can toggle multiple bits in any stream of numbers. This can include setting
    /// bits `1 - 3` to `true`, or setting bits `2-7` to `false`.
    ///
    /// # Safety
    /// `set_bit_range` has protections for when you try to set a bit outside the range of the type
    /// you are setting. For example, if you try to set bit `14` in a `u8` type, this will panic as
    /// bit `14` does not exist in a u8 type.
    ///
    /// # Portability
    /// The `set_bit_range` trait can be implemented for any type as long as the type contains a stream
    /// of numbers ( without gaps ) that the user would like to toggle. This could be a custom struct
    /// that contains a primitive type.
    ///
    /// # Use
    /// ```rust
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.set_bit_range(0..2, 0b01);
    /// assert_eq!(value, 17);
    ///
    /// ```
    fn set_bit_range(&mut self, r: Range<u8>, bits: u64) -> Self {
        let bit_out_of_range = bits as u64 / (2 as u64).pow(r.end as u32);
        if bit_out_of_range > 0 { panic!("Tried to set a bit outside the range provided!"); }

        let mut dev = 0;
        for bit in r {
            self.set_bit(bit, (bits >> dev & 1) == 1);

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
    /// use quantum_os::bitset::BitSet;
    ///
    /// let value = 16_u8.get_bit(0);
    /// assert_eq!(value, 0);
    ///
    /// ```
    fn get_bit(&self, bit: u8) -> Self {
        use core::mem::size_of;

        let self_size_bits = size_of::<Self>() * 8;
        if bit > self_size_bits as u8 { panic!("Tried to get a bit outside the range of the current type!"); }

        (self >> bit) & 0b1
    }
}

#[cfg(test)]
mod test_case {
    use crate::bitset::BitSet;

    #[test_case]
    fn test_setbit_u8() {

        let mut initial_value = 10_u8;
        let output_value = 14_u8;

        let value = initial_value.set_bit(2, true);

        assert_eq!(value, output_value);
        assert_eq!(10_u8.set_bit_range(0..8, 255), 255);
        assert_eq!(0_u8.set_bit(0, true), 1);
        assert_eq!(8_u8.set_bit(3, false), 0);
        assert_eq!(0_u8.set_bit_range(2..3, 1), 0_u8.set_bit(2, true));
    }

    #[test_case]
    fn test_all() {
        assert_eq!(100.get_bit(2), 1);
        assert_eq!(12_u8.set_bit_range(3..5, 0b10), 20);
        assert_eq!(10_u16.set_bit_range(0..8, 255), 255);
        assert_eq!(0_u32.set_bit(0, true), 1);
        assert_eq!(8_u64.set_bit(3, false), 0);
        assert_eq!(0.set_bit_range(2..3, 1), 0_u8.set_bit(2, true));
    }
}