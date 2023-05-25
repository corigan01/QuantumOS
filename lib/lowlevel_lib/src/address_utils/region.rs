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


use core::any::{type_name};
use core::fmt::{Debug, Formatter};
use crate::address_utils::addressable::Addressable;
use crate::bytes::Bytes;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MemoryRegionType {
    Usable,
    KernelCode,
    Reserved,
    Bios,
    Uefi,
    UnavailableMemory,
    Unknown
}

#[derive(Clone, Copy)]
pub struct MemoryRegion<Type = u64> {
    start: Type,
    end: Type,
    region_type: MemoryRegionType,
}

impl<Type> MemoryRegion<Type>
    where Type: Addressable + Copy {

    pub fn new(start: Type, end: Type, region_type: MemoryRegionType) -> Self {
        assert!(start.address_as_u64() < end.address_as_u64());
        Self {
            start,
            end,
            region_type
        }
    }

    pub fn from_distance(start: Type, distance: Bytes, region_type: MemoryRegionType) -> Self {
        Self::new(start, start.copy_by_offset(distance.into()), region_type)
    }

    pub fn size(&self) -> u64 {
        self.start.distance_from_address(&self.end)
    }

    pub fn get_start_address(&self) -> &Type {
        &self.start
    }

    pub fn get_end_address(&self) -> &Type {
        &self.end
    }

    pub fn region_type(&self) -> MemoryRegionType {
        self.region_type
    }

    pub fn bytes(&self) -> Bytes {
        Bytes::from(self.size())
    }
}

impl<Type> Debug for MemoryRegion<Type>
    where Type: Debug + Addressable + Copy {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let type_name = type_name::<Type>().split("::").last().unwrap_or("");
        if f.alternate() {
            write!(f, "MemoryRegion<{}> {{\n    type:  {:?}\n    start: 0x{:x},\n    end:   0x{:x},\n    size:  {}\n}}", type_name, self.region_type, self.start.address_as_u64(), self.end.address_as_u64(), Bytes::from(self.size()))
        } else {
            write!(f, "MemoryRegion<{}> {{ type: {:?}, start: 0x{:x}, end: 0x{:x}, size: {} }}", type_name,  self.region_type, self.start.address_as_u64(), self.end.address_as_u64(), Bytes::from(self.size()))
        }
    }
}
