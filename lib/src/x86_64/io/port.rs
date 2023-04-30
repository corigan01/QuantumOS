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

use crate::x86_64::raw_cpu_io_port::{byte_in, byte_out, word_in, word_out};

pub struct ReadWritePort;
pub struct WriteOnlyPort;
pub struct ReadOnlyPort;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct IOPort(u16);

impl IOPort {
    pub fn new(n: u16) -> IOPort {
        IOPort ( n )
    }

    pub fn as_u16(&self) -> u16 {
        self.0
    }

    pub fn mutate_offset_by(&mut self, offset: i16) {
        self.0 = (offset as i32 + self.0 as i32) as u16;
    }
    
    pub fn clone_from_offset_by(&self, offset: i16) -> Self {
        let mut cloned_value = self.clone();
        cloned_value.mutate_offset_by(offset);
        
        cloned_value
    }

    pub unsafe fn read_u8(&self) -> u8 {
        byte_in(self.0)
    }

    pub unsafe fn read_u16(&self) -> u16 {
        word_in(self.0)
    }

    pub unsafe fn write_u8(&self, data: u8) {
        byte_out(self.0, data);
    }

    pub unsafe fn write_u16(&self, data: u16) {
        word_out(self.0, data);
    }
}
