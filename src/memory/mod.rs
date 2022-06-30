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

use core::result;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtualAddress(u64);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct NotValidAddress(u64);

impl NotValidAddress {
    pub fn get_ptr(self) -> u64 {
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
        match address & (0xFF00000000000000 as u64) {
            0 => Ok(VirtualAddress(address)),
            _ => Err(NotValidAddress(address))
        }
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


    // unsafe
    pub unsafe fn new_unsafe(address: u64) -> VirtualAddress { VirtualAddress(address) }
}