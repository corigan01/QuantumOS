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

use core::ptr::null;
use crate::debug_println;
use crate::memory::VirtualAddress;

#[derive(Debug, Clone, Copy)]
pub struct SafePtr<T> {
    pointer: Option<*const T>,
}

impl<T> SafePtr<T> {
    pub fn new() -> Self {
        Self {
            pointer: None
        }
    }

    pub fn new_from_ptr(ptr: *mut T) -> Self {
        if ptr.is_null() {
            return Self::new();
        }
        
        unsafe { Self::unsafe_new(ptr) }
    }

    pub unsafe fn unsafe_from_address(ptr: VirtualAddress) -> Self {
        Self::new_from_ptr(ptr.as_u64() as *mut T)
    }

    pub unsafe fn unsafe_new(ptr: *mut T) -> Self {
        Self {
            pointer: Some(ptr)
        }
    }

    pub unsafe fn advance_ptr(&self) -> Option<*mut T> {
        if !self.is_valid() {
            return None;
        }

        Some((self.pointer.unwrap() as *mut T).offset(1))
    }

    pub fn as_ptr(&self) -> Option<*mut T> {
        if let Some(pointer) = self.pointer {
            Some(pointer as *mut _)
        }
        else {
            None
        }
    }

    pub fn is_valid(&self) -> bool {
        self.pointer.is_some()
    }
}

pub fn test() {
    use crate::arch_x86_64::idt::Entry;
    use crate::arch_x86_64::idt::EntryOptions;

    let something: SafePtr<i32> =
        unsafe { SafePtr::unsafe_from_address(VirtualAddress::from_ptr(&mut 8)) };

    if let Some(number) = something.as_ptr() {
        unsafe { debug_println!("Value: {:?} *{:?}", *number, number); };
    } else {
        debug_println!("NULL PTR!!");
    }


}