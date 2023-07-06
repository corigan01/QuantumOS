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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MemoryRegionType {
    Unknown = 0,
    Usable,
    KernelCode,
    KernelStack,
    BootInfo,
    Reserved,
    Bios,
    Uefi,
    UnavailableMemory,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum HowOverlapping {
    EndsIn,
    StartsIn,
    Within,
    OverExpands,
    None
}

#[derive(Clone, Copy)]
pub struct MemoryRegion<Type = u64> {
    start: Type,
    end: Type,
    region_type: MemoryRegionType,
}

impl<Type> PartialEq for MemoryRegion<Type>
    where Type: Addressable + Copy {
    fn eq(&self, other: &Self) -> bool {
        self.start.address_as_u64() == other.start.address_as_u64() &&
            self.end.address_as_u64() == other.end.address_as_u64() &&
            self.region_type == other.region_type
    }
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


    pub fn how_overlapping(&self, rhs: &MemoryRegion<Type>) -> HowOverlapping {
        let self_start = self.start.address_as_u64();
        let self_end = self.end.address_as_u64();

        let other_start = rhs.start.address_as_u64();
        let other_end = rhs.end.address_as_u64();


        if other_start > self_start && other_end < self_end {
            HowOverlapping::Within
        } else if other_start > self_start && other_start < self_end && other_end >= self_end {
            HowOverlapping::StartsIn
        } else if other_start <= self_start && other_end < self_end && other_end >= self_start {
            HowOverlapping::EndsIn
        } else if other_start <= self_start && other_end >= self_end {
            HowOverlapping::OverExpands
        } else {
            HowOverlapping::None
        }
    }

    pub fn is_usable(&self) -> bool {
        self.region_type == MemoryRegionType::Usable
    }

    pub fn is_reserved(&self) -> bool {
        self.region_type == MemoryRegionType::Reserved
    }

    pub fn is_kernel(&self) -> bool {
        self.region_type == MemoryRegionType::KernelStack ||
            self.region_type == MemoryRegionType::KernelCode
    }

    pub fn size(&self) -> u64 {
        self.start.distance_from_address(&self.end)
    }

    pub const fn get_start_address(&self) -> &Type {
        &self.start
    }

    pub const fn get_end_address(&self) -> &Type {
        &self.end
    }

    pub const fn region_type(&self) -> MemoryRegionType {
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

#[cfg(test)]
mod test {
    use crate::address_utils::region::{HowOverlapping, MemoryRegion, MemoryRegionType};

    #[test]
    fn test_contained_overlapping() {
        let region =
            MemoryRegion::new(0, 100, MemoryRegionType::Usable);

        let other =
            MemoryRegion::new(50, 70, MemoryRegionType::Reserved);

        assert_eq!(region.how_overlapping(&other), HowOverlapping::Within);
    }

    #[test]
    fn test_starts_in_overlapping() {
        let region =
            MemoryRegion::new(0, 100, MemoryRegionType::Usable);

        let other =
            MemoryRegion::new(80, 120, MemoryRegionType::Reserved);

        assert_eq!(region.how_overlapping(&other), HowOverlapping::StartsIn);

        let other =
            MemoryRegion::new(80, 100, MemoryRegionType::Reserved);

        assert_eq!(region.how_overlapping(&other), HowOverlapping::StartsIn);
    }

    #[test]
    fn test_ends_in_overlapping() {
        let region =
            MemoryRegion::new(70, 200, MemoryRegionType::Usable);

        let other =
            MemoryRegion::new(50, 100, MemoryRegionType::Reserved);

        assert_eq!(region.how_overlapping(&other), HowOverlapping::EndsIn);

        let other =
            MemoryRegion::new(50, 70, MemoryRegionType::Reserved);

        assert_eq!(region.how_overlapping(&other), HowOverlapping::EndsIn);
    }

    #[test]
    fn test_over_expands_overlapping() {
        let region =
            MemoryRegion::new(50, 100, MemoryRegionType::Usable);

        let other =
            MemoryRegion::new(40, 200, MemoryRegionType::Reserved);

        assert_eq!(region.how_overlapping(&other), HowOverlapping::OverExpands);

        let other =
            MemoryRegion::new(50, 100, MemoryRegionType::Reserved);

        assert_eq!(region.how_overlapping(&other), HowOverlapping::OverExpands);
    }

    #[test]
    fn test_none_overlapping() {
        let region =
            MemoryRegion::new(50, 100, MemoryRegionType::Usable);

        let other =
            MemoryRegion::new(120, 200, MemoryRegionType::Reserved);

        assert_eq!(region.how_overlapping(&other), HowOverlapping::None);
    }


}