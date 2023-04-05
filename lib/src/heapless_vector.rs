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

pub enum HeaplessVecErr {
    VectorFull,
    OutOfBounds,
}

pub struct HeaplessVec<Type, const SIZE: usize> {
    internal_data: [Type; SIZE],
    used_data: usize,
}

impl<Type, const SIZE: usize> HeaplessVec<Type, SIZE>
where
    Type: Default,
    Type: Copy,
{
    pub fn new() -> Self {
        Self {
            internal_data: [Default::default(); SIZE],
            used_data: 0,
        }
    }

    pub fn push_within_capsity(&mut self, value: Type) -> Result<(), HeaplessVecErr> {
        if self.used_data >= self.internal_data.len() {
            return Err(HeaplessVecErr::VectorFull);
        }

        let index = self.used_data;
        self.internal_data[index] = value;

        self.used_data += 1;

        Ok(())
    }

    pub fn pop(&mut self) -> Option<Type> {
        if self.used_data == 0 {
            return None;
        }

        self.used_data -= 1;

        Some(self.internal_data[self.used_data + 1])
    }

    pub fn get<'a>(&'a self, index: usize) -> Result<&'a Type, HeaplessVecErr> {
        if index > self.used_data {
            return Err(HeaplessVecErr::OutOfBounds);
        }

        return Ok(&self.internal_data[index]);
    }

    pub fn get_mut<'a>(&'a mut self, index: usize) -> Result<&'a mut Type, HeaplessVecErr> {
        if index > self.used_data {
            return Err(HeaplessVecErr::OutOfBounds);
        }

        return Ok(&mut self.internal_data[index]);
    }

    pub fn as_slice<'a>(&'a self) -> &'a [Type] {
        self.internal_data.as_slice()
    }

    pub fn as_mut_slice<'a>(&'a mut self) -> &'a mut [Type] {
        self.internal_data.as_mut_slice()
    }

    pub fn as_ptr(&self) -> *const Type {
        return self.internal_data.as_ptr();
    }

    pub fn as_mut_ptr(&mut self) -> *mut Type {
        return self.internal_data.as_mut_ptr();
    }
}
