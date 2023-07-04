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

use core::mem::{align_of, size_of};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MemoryLayout {
    alignment: usize, 
    bytes: usize
}

impl MemoryLayout {
    pub fn new(alignment: usize, bytes: usize) -> Self {
        Self {
            alignment,
            bytes
        }
    }
    
    pub fn from_type<T>() -> Self {
        let alignment = align_of::<T>();
        let size = size_of::<T>();
        
        Self::new(alignment, size)
    }

    pub fn from_type_array<T>(elements: usize) -> Self {
        let alignment = align_of::<T>();
        let size = size_of::<T>() * elements;

        Self::new(alignment, size)
    }
    
    pub fn alignment(&self) -> usize {
        self.alignment
    }
    
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}