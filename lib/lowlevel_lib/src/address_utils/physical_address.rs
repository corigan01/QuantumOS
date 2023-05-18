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

use crate::address_utils::addressable::Addressable;
use crate::address_utils::{InvdAddress, PHYSICAL_ALLOWED_ADDRESS_SIZE};
use crate::bitset::BitSet;
use core::marker::PhantomData;
use core::ops::{RangeBounds, RangeFull};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Unaligned {}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Aligned {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PhyAddress<Type = Unaligned, const ALIGNED_BITS: u64 = 12> {
    value: u64,
    reserved: PhantomData<Type>,
}

impl PhyAddress {
    fn x(addr: u64) -> PhyAddress {
        Self {
            value: addr,
            reserved: Default::default(),
        }
    }

    pub fn new(addr: u64) -> Result<PhyAddress, InvdAddress> {
        if Self::is_address_physical_compatible(addr) {
            return Ok(PhyAddress::x(addr));
        }

        Err(InvdAddress(addr))
    }

    pub fn try_new(addr: u64) -> Option<PhyAddress> {
        if Self::is_address_physical_compatible(addr) {
            return Some(PhyAddress::x(addr));
        }

        None
    }

    pub fn is_address_physical_compatible(address: u64) -> bool {
        address.get_bits(PHYSICAL_ALLOWED_ADDRESS_SIZE..64)
            == u64::MAX.get_bits(PHYSICAL_ALLOWED_ADDRESS_SIZE..64)
            || address.get_bits(PHYSICAL_ALLOWED_ADDRESS_SIZE..64) == 0
    }

    pub fn is_address_aligned(address: u64, alignment: usize) -> bool {
        address & (2_u64.pow(alignment as u32) - 1) == 0
    }

    pub fn strip_unaligned_bits_to_align_address<const ALIGNED_BITS: u64>(
        mut self,
    ) -> PhyAddress<Aligned, ALIGNED_BITS> {
        PhyAddress {
            value: self.value.set_bits(0..((ALIGNED_BITS + 1) as u8), 0),
            reserved: Default::default(),
        }
    }

    pub fn try_aligned<const ALIGNED_BITS: u64>(
        self,
    ) -> Result<PhyAddress<Aligned, ALIGNED_BITS>, InvdAddress> {
        if Self::is_address_aligned(self.value, ALIGNED_BITS as usize) {
            return Ok(PhyAddress {
                value: self.value,
                reserved: Default::default(),
            });
        }

        Err(InvdAddress(self.value))
    }
}
impl<Any, const ALIGNED_BITS: u64> PhyAddress<Any, ALIGNED_BITS> {
    pub fn is_null(&self) -> bool {
        self.value == 0
    }

    pub fn is_some(&self) -> bool {
        self.value != 0
    }

    pub fn is_in_range(&self, range: RangeFull) -> bool {
        range.contains(&self.value)
    }

    pub fn to_ptr<T>(&self) -> *const T {
        self.value as *const T
    }

    pub fn to_mut_ptr<T>(&mut self) -> *mut T {
        self.value as *mut T
    }

    pub fn as_u64(&self) -> u64 {
        self.value
    }

    pub fn is_32_bit_compatible(&self) -> bool {
        self.value.get_bits(33..64) > 0
    }
}

impl Addressable for PhyAddress {
    fn address_as_u64(&self) -> u64 {
        self.as_u64()
    }

    fn copy_by_offset(&self, distance: u64) -> Self {
        PhyAddress::new(self.as_u64() + distance).unwrap()
    }
}

macro_rules! from_all_types {
    ($($t:ty)*) => ($(
        impl TryFrom<$t> for PhyAddress<Unaligned> {
            type Error = InvdAddress;

            fn try_from(value: $t) -> Result<Self, Self::Error> {
                PhyAddress::new(value as u64)
            }
        }
    )*)
}

macro_rules! to_all_types {
    ($($t:ty)*) => ($(
        impl<Any, const ALIGNED_BITS: u64> TryInto<$t> for PhyAddress<Any, ALIGNED_BITS> {
            type Error = InvdAddress;

            fn try_into(self) -> Result<$t, Self::Error> {
                let value = self.as_u64();

                if value > (<$t>::MAX as u64) {
                    return Err(InvdAddress(value));
                }

                return Ok(value as $t)
            }
        }
    )*)
}

to_all_types! { u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 usize isize }
from_all_types! { u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 usize isize }
