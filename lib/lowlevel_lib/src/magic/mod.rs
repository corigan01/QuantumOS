/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
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

use core::fmt::Debug;

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct TransferableDataProtector<T> {
    data: T,
    safety_magic: u64,
}

pub enum MagicType {
    BuiltinMagic,
    CustomMagic(u64),
}

impl MagicType {
    const BUILT_IN_MAGIC: u64 = 0x5175616E74756D;

    pub fn to_magic(&self) -> u64 {
        match self {
            Self::BuiltinMagic => Self::BUILT_IN_MAGIC,
            Self::CustomMagic(value) => *value,
        }
    }

    pub fn from_magic(magic: u64) -> Self {
        match magic {
            Self::BUILT_IN_MAGIC => Self::BuiltinMagic,
            v => Self::CustomMagic(v),
        }
    }
}

impl PartialEq for MagicType {
    fn eq(&self, other: &Self) -> bool {
        let ours = self.to_magic();
        let theirs = other.to_magic();

        ours == theirs
    }
}

impl<T> TransferableDataProtector<T>
where
    T: Clone + Copy + Debug,
{
    pub fn special_transfer(data: T, magic_type: MagicType) -> Self {
        let safety_magic = magic_type.to_magic();

        Self { data, safety_magic }
    }

    pub fn transfer(data: T) -> Self {
        Self::special_transfer(data, MagicType::BuiltinMagic)
    }

    pub unsafe fn collapse(ptr: u64) -> Option<T> {
        if ptr == 0 {
            return None;
        }

        let typed_ptr = ptr as *const Self;
        let value = *typed_ptr;

        if MagicType::from_magic(value.safety_magic) != MagicType::BuiltinMagic {
            return None;
        }

        Some(value.data)
    }

    pub unsafe fn special_collapse(ptr: u64, expected_magic: MagicType) -> Option<T> {
        if ptr == 0 {
            return None;
        }

        let typed_ptr = ptr as *const Self;
        let value = *typed_ptr;

        if MagicType::from_magic(value.safety_magic) != expected_magic {
            return None;
        }

        Some(value.data)
    }
}

pub struct DataProtector {
    magic: u64,
}

impl DataProtector {
    pub fn new() -> Self {
        Self {
            magic: MagicType::BuiltinMagic.to_magic(),
        }
    }

    pub fn is_magic_valid(&self) -> bool {
        MagicType::from_magic(self.magic) == MagicType::BuiltinMagic
    }

    pub fn is_special_magic_valid(&self, magic: MagicType) -> bool {
        MagicType::from_magic(self.magic) == magic
    }

    pub unsafe fn collapse_ptr<T>(&self, ptr: u64) -> Option<T>
    where
        T: Clone + Copy,
    {
        if !self.is_magic_valid() {
            return None;
        }

        Some(*(ptr as *const T))
    }

    pub unsafe fn collapse_ptr_ref<T>(&self, ptr: u64) -> Option<&T>
    where
        T: Clone + Copy,
    {
        if !self.is_magic_valid() {
            return None;
        }

        Some(&*(ptr as *const T))
    }

    pub unsafe fn special_collapse_ptr<T>(&self, magic: MagicType, ptr: u64) -> Option<T>
    where
        T: Clone + Copy,
    {
        if !self.is_special_magic_valid(magic) {
            return None;
        }

        Some(*(ptr as *const T))
    }

    pub unsafe fn special_collapse_ptr_ref<T>(&self, magic: MagicType, ptr: u64) -> Option<&T>
    where
        T: Clone + Copy,
    {
        if !self.is_special_magic_valid(magic) {
            return None;
        }

        Some(&*(ptr as *const T))
    }
}
