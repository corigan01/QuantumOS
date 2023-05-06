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

pub trait Addressable {
    fn address_as_u64(&self) -> u64;

    fn copy_by_offset(&self, distance: u64) -> Self;

    fn distance_from_address(&self, addr: &Self) -> u64 {
        let our_value = self.address_as_u64();
        let rhs_value = addr.address_as_u64();

        our_value.abs_diff(rhs_value)
    }
}


macro_rules! impl_all_types {
    ($($t:ty)*) => ($(
        impl Addressable for $t {
            fn address_as_u64(&self) -> u64 {
                *self as u64
            }

            fn copy_by_offset(&self, distance: u64) -> $t {
                (self.address_as_u64() + distance) as $t
            }
        }
    )*)
}

impl_all_types! { u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 usize isize }
