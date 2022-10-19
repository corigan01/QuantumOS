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
furnished to do so, subject to the following conditions

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/


use crate::bitset::BitSet;

pub mod paging;
pub mod physical_memory;
pub mod pmm;
pub mod heap;
pub mod init_alloc;

const PAGE_SIZE: usize = 4096;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum UsedMemoryKind {
    Unknown   = 0,
    Kernel    = 1,
    UserSpace = 2,
    Volatile  = 3,
}

impl UsedMemoryKind {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Kernel,
            2 => Self::UserSpace,
            3 => Self::Volatile,
            _ => Self::Unknown
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct VirtualAddress(u64);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct PhysicalAddress(u64);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct NotValidAddress(u64);

impl NotValidAddress {
    pub fn get_address(self) -> u64 {
        self.0
    }
}

// Only bits under 48-64 are valid
impl VirtualAddress {

    #[inline]
    pub fn new(address: u64) -> VirtualAddress {
        VirtualAddress::try_new(address).expect("Not Valid Address, bit check failed!")
    }

    #[inline]
    pub fn try_new(address: u64) -> Result<VirtualAddress, NotValidAddress> {
        if address.get_bits(48..64) == u64::MAX.get_bits(48..64) ||
            address.get_bits(48..64) == 0 {
            return Ok(VirtualAddress(address));
        }

        Err(NotValidAddress(address))
    }

    #[inline]
    pub fn zero() -> VirtualAddress { VirtualAddress(0) }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.as_u64() == 0
    }

    #[inline]
    pub fn is_some(&self) -> bool {
        self.as_u64() > 0
    }

    #[inline]
    pub fn from_ptr<T>(ptr: *const T) -> VirtualAddress {
        VirtualAddress::new(ptr as u64)
    }

    #[inline]
    pub fn as_ptr<T>(self) -> *const T {
        self.as_u64() as *const T
    }

    #[inline]
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.as_u64() as *mut T
    }

    /// # Safety
    /// Using this function means that the user must ensure that the input is valid and doesnt
    /// cause fault.
    pub unsafe fn new_unsafe(address: u64) -> VirtualAddress { VirtualAddress(address) }
}

impl PhysicalAddress {
    #[inline]
    pub fn new(address: u64) -> PhysicalAddress {
        PhysicalAddress::try_new(address).expect("Not Valid Address, bit check failed!")
    }

    #[inline]
    pub fn try_new(address: u64) -> Result<PhysicalAddress, NotValidAddress> {
        Ok(PhysicalAddress(address))
    }

    #[inline]
    pub fn zero() -> PhysicalAddress { PhysicalAddress(0) }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.as_u64() == 0
    }

    #[inline]
    pub fn is_some(&self) -> bool {
        self.as_u64() > 0
    }

    #[inline]
    pub fn from_ptr<T>(ptr: *const T) -> PhysicalAddress {
        PhysicalAddress::new(ptr as u64)
    }

    #[inline]
    pub fn as_ptr<T>(self) -> *const T {
        self.as_u64() as *const T
    }

    #[inline]
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.as_u64() as *mut T
    }

    /// # Safety
    /// This creates a new PhysicalAddress without checking if the address is valid. This means
    /// that the user must ensure that the address contains a valid address that will not cause a
    /// fault!
    pub unsafe fn new_unsafe(address: u64) -> PhysicalAddress { PhysicalAddress(address) }
}


#[cfg(test)]
mod test_case {
    use crate::bitset::BitSet;
    use crate::memory::VirtualAddress;

    #[test_case]
    fn test_virtual_address() {
        let address_number = 1002040_u64;
        let address = VirtualAddress::new(address_number);

        assert_eq!(address.as_u64(), address_number);
        assert_eq!(address.is_null(), false);
        assert_eq!(address.is_some(), true);
    }

    #[test_case]
    fn try_fail_creation() {
        let address_number = u64::MAX;
        let address = VirtualAddress::try_new(address_number);

        assert_eq!(address.is_ok(), true);
        assert_eq!(address.unwrap().as_u64(), address_number);

        let address_number = u64::MAX.set_bit(49, false);
        let address = VirtualAddress::try_new(address_number);

        assert_eq!(address.is_ok(), false);
        assert_eq!(address.is_err(), true);
        assert_eq!(address.err().unwrap().get_address(), address_number);
    }


}