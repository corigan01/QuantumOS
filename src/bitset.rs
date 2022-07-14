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

#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]


trait BitSet {
    fn setbit(&mut self, bit: u8, v: bool) -> Self;
    fn getbit(&self, bit: u8) -> Self;
}

impl BitSet for u8 {
    fn setbit(&mut self, bit: u8, v: bool) -> Self {
        use core::mem::size_of;

        let self_size_bits = size_of::<Self>() * 8;
        if bit > self_size_bits as Self { panic!("Tried to set a bit outside the range of the current type!"); }

        let calculate_bit = (2 as Self).pow(bit as u32);
        *self = if v { *self | calculate_bit } else { *self & ( calculate_bit ^ Self::MAX) };

        *self
    }

    fn getbit(&self, bit: u8) -> Self {
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

        let value = initial_value.setbit(2, true);

        assert_eq!(value, output_value);
    }
}