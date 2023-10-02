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

use core::mem::size_of;
use core::ptr;
use crate::error::FsError;
use crate::filesystems::dosfs::structures::DoubleWord;

#[repr(C, packed)]
pub struct FsInfo {
    leading_signature: DoubleWord,
    // todo: remove this from the structure and update loading to expect but not load its value
    reserved1: [u8; 480],
    structure_signature: DoubleWord,
    free_count: DoubleWord,
    next_free: DoubleWord,
    reserved2: [u8; 12],
    trail_signature: DoubleWord
}

impl FsInfo {
    const LEADING_SIGNATURE: DoubleWord = 0x41615252;
    const STRUCTURE_SIGNATURE: DoubleWord = 0x61417272;
    const TRAIL_SIGNATURE: DoubleWord = 0xAA550000;

    pub fn is_structure_valid(&self) -> bool {
        self.leading_signature == Self::LEADING_SIGNATURE &&
            self.structure_signature == Self::STRUCTURE_SIGNATURE &&
            self.trail_signature == Self::TRAIL_SIGNATURE
    }

    pub fn free_clusters(&self) -> Option<usize> {
        if self.free_count == 0xFFFFFFFF {
            return None;
        }

        Some(self.free_count as usize)
    }

    pub fn next_free(&self) -> Option<usize> {
        if self.next_free == 0xFFFFFFFF {
            return None;
        }

        Some(self.next_free as usize)
    }
}

impl TryFrom<&[u8]> for FsInfo {
    type Error = FsError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < size_of::<Self>() {
            return Err(FsError::try_from_array_error::<Self>(value));
        }

        Ok(unsafe { ptr::read(value.as_ptr() as *const Self) })
    }
}

