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

use core::fmt::{Debug, Formatter};

use crate::address_utils::addressable::Addressable;
use crate::address_utils::region::{HowOverlapping, MemoryRegion, MemoryRegionType};
use crate::bytes::Bytes;
use crate::heapless_vector::{HeaplessVec, HeaplessVecErr};

const MAX_ALLOCATABLE_REGIONS: usize = 40;

pub struct RegionMap<Type> {
    regions: HeaplessVec<MemoryRegion<Type>, MAX_ALLOCATABLE_REGIONS>,
}

impl<Type> RegionMap<Type>
    where Type: Addressable + Copy {
    pub fn new() -> Self {
        Self {
            regions: Default::default()
        }
    }

    // TODO: Maybe there is a better way of doing this in the future
    pub fn consolidate(&mut self) -> Result<(), HeaplessVecErr> {
        loop {
            let mut new_free_regions: HeaplessVec<MemoryRegion<Type>, MAX_ALLOCATABLE_REGIONS> = HeaplessVec::new();

            let usable_iterator =
                self.regions.iter().filter(|region|
                    region.region_type() == MemoryRegionType::Usable
                );

            let mut was_work_done = false;

            'outer: for free_regions in usable_iterator {
                let non_usable_iterator =
                    self.regions.iter().filter(|region|
                        region.region_type() != MemoryRegionType::Usable
                    );

                let free_start = free_regions.get_start_address().address_as_u64();

                for all_other_regions in non_usable_iterator {
                    let overlapping_status = free_regions.how_overlapping(all_other_regions);

                    let other_start = all_other_regions.get_start_address().address_as_u64();

                    match overlapping_status {
                        HowOverlapping::EndsIn => {
                            let shrunk_region = MemoryRegion::new(
                                all_other_regions.get_end_address().copy_by_offset(1),
                                *free_regions.get_end_address(),
                                free_regions.region_type(),
                            );

                            new_free_regions.push_within_capacity(shrunk_region)?;

                            was_work_done = true;
                            break 'outer;
                        }
                        HowOverlapping::StartsIn => {
                            let shrunk_region = MemoryRegion::new(
                                *free_regions.get_start_address(),
                                free_regions.get_start_address().copy_by_offset((other_start - free_start) - 1),
                                free_regions.region_type(),
                            );

                            new_free_regions.push_within_capacity(shrunk_region)?;

                            was_work_done = true;
                            break 'outer;
                        }
                        HowOverlapping::Within => {
                            let before_region = MemoryRegion::new(
                                *free_regions.get_start_address(),
                                free_regions.get_start_address().copy_by_offset((other_start - free_start) - 1),
                                free_regions.region_type(),
                            );

                            let after_region = MemoryRegion::new(
                                all_other_regions.get_end_address().copy_by_offset(1),
                                *free_regions.get_end_address(),
                                free_regions.region_type(),
                            );

                            new_free_regions.push_within_capacity(before_region)?;
                            new_free_regions.push_within_capacity(after_region)?;

                            was_work_done = true;
                            break 'outer;
                        }
                        HowOverlapping::OverExpands => {
                            was_work_done = true;
                            break 'outer;
                        }
                        HowOverlapping::None => {}
                    }

                    new_free_regions.push_within_capacity(*free_regions)?;
                }
            }

            if !was_work_done {
                break;
            }

            self.regions.retain(|region| {
                region.region_type() != MemoryRegionType::Usable
            });

            self.regions.push_vec(new_free_regions)?;
        }


        Ok(())
    }

    pub fn add_new_region(&mut self, value: MemoryRegion<Type>) -> Result<(), HeaplessVecErr> {
        self.regions.push_within_capacity(value)
    }

    pub fn run_on_type<Function>(&self, t: MemoryRegionType, runner: &mut Function)
        where Function: FnMut(&MemoryRegion<Type>) {
        for region in self.regions.iter() {
            let region_type = region.region_type();

            if t == region_type {
                runner(region);
            }
        }
    }

    pub fn total_mem(&self) -> Bytes {
        let mut total_bytes = Bytes::from(0);
        for region in self.regions.iter() {
            total_bytes += region.bytes();
        }

        total_bytes
    }

    pub fn total_mem_for_type(&self, t: MemoryRegionType) -> Bytes {
        let mut total_bytes = Bytes::from(0);
        for region in self.regions.iter() {
            if region.region_type() == t {
                total_bytes += region.bytes();
            }
        }

        total_bytes
    }
}

impl<Type> Debug for RegionMap<Type>
    where Type: Addressable + Debug + Copy {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for (index, region) in self.regions.iter().enumerate() {
            writeln!(f, "[{index}]: {region:?}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::address_utils::physical_address::PhyAddress;
    use crate::address_utils::region::{MemoryRegion, MemoryRegionType};
    use crate::address_utils::region_map::RegionMap;

    #[test]
    fn test_within_consolidation() {
        let mut region_map = RegionMap::new();

        let free_region = MemoryRegion::new(
            PhyAddress::new(0x000).unwrap(),
            PhyAddress::new(0x1000).unwrap(),
            MemoryRegionType::Usable,
        );

        let kernel_code_region = MemoryRegion::new(
            PhyAddress::new(0x050).unwrap(),
            PhyAddress::new(0x0090).unwrap(),
            MemoryRegionType::KernelCode,
        );

        region_map.add_new_region(free_region).unwrap();
        region_map.add_new_region(kernel_code_region).unwrap();

        region_map.consolidate().unwrap();

        // TODO: We are not sure which region will be first, so maybe don't hard code this later
        let not_free_region = *region_map.regions.get(0).unwrap();
        let free_1_region = *region_map.regions.get(1).unwrap();
        let free_2_region = *region_map.regions.get(2).unwrap();

        assert_eq!(region_map.regions.len(), 3);

        assert_eq!(
            not_free_region,
            kernel_code_region
        );

        assert_eq!(free_1_region,
                   MemoryRegion::new(
                       PhyAddress::new(0).unwrap(),
                       PhyAddress::new(0x4f).unwrap(),
                       MemoryRegionType::Usable,
                   )
        );

        assert_eq!(free_2_region,
                   MemoryRegion::new(
                       PhyAddress::new(0x91).unwrap(),
                       PhyAddress::new(0x1000).unwrap(),
                       MemoryRegionType::Usable,
                   )
        );
    }

    #[test]
    fn test_ends_in_consolidation() {
        let mut region_map = RegionMap::new();

        let free_region = MemoryRegion::new(
            PhyAddress::new(0x500).unwrap(),
            PhyAddress::new(0x1000).unwrap(),
            MemoryRegionType::Usable,
        );

        let reserved_region = MemoryRegion::new(
            PhyAddress::new(0x400).unwrap(),
            PhyAddress::new(0x0600).unwrap(),
            MemoryRegionType::Reserved,
        );

        region_map.add_new_region(free_region).unwrap();
        region_map.add_new_region(reserved_region).unwrap();

        region_map.consolidate().unwrap();

        // TODO: We are not sure which region will be first, so maybe don't hard code this later
        let not_free_region = *region_map.regions.get(0).unwrap();
        let free_1_region = *region_map.regions.get(1).unwrap();

        assert_eq!(region_map.regions.len(), 2);

        assert_eq!(not_free_region, reserved_region);

        assert_eq!(free_1_region,
                   MemoryRegion::new(
                       PhyAddress::new(0x601).unwrap(),
                       PhyAddress::new(0x1000).unwrap(),
                       MemoryRegionType::Usable,
                   )
        );
    }

    #[test]
    fn test_starts_in_consolidation() {
        let mut region_map = RegionMap::new();

        let free_region = MemoryRegion::new(
            PhyAddress::new(0x500).unwrap(),
            PhyAddress::new(0x1000).unwrap(),
            MemoryRegionType::Usable,
        );

        let reserved_region = MemoryRegion::new(
            PhyAddress::new(0x900).unwrap(),
            PhyAddress::new(0x2000).unwrap(),
            MemoryRegionType::Reserved,
        );

        region_map.add_new_region(free_region).unwrap();
        region_map.add_new_region(reserved_region).unwrap();

        region_map.consolidate().unwrap();

        // TODO: We are not sure which region will be first, so maybe don't hard code this later
        let not_free_region = *region_map.regions.get(0).unwrap();
        let free_1_region = *region_map.regions.get(1).unwrap();

        assert_eq!(region_map.regions.len(), 2);

        assert_eq!(not_free_region, reserved_region);

        assert_eq!(free_1_region,
                   MemoryRegion::new(
                       PhyAddress::new(0x500).unwrap(),
                       PhyAddress::new(0x8ff).unwrap(),
                       MemoryRegionType::Usable,
                   )
        );
    }

    #[test]
    fn test_over_expands_consolidation() {
        let mut region_map = RegionMap::new();

        let free_region = MemoryRegion::new(
            PhyAddress::new(0x500).unwrap(),
            PhyAddress::new(0x1000).unwrap(),
            MemoryRegionType::Usable,
        );

        let reserved_region = MemoryRegion::new(
            PhyAddress::new(0x400).unwrap(),
            PhyAddress::new(0x2000).unwrap(),
            MemoryRegionType::Reserved,
        );

        region_map.add_new_region(free_region).unwrap();
        region_map.add_new_region(reserved_region).unwrap();

        region_map.consolidate().unwrap();

        // TODO: We are not sure which region will be first, so maybe don't hard code this later
        let not_free_region = *region_map.regions.get(0).unwrap();

        assert_eq!(region_map.regions.len(), 1);

        assert_eq!(not_free_region, reserved_region);
    }

    #[test]
    fn test_none_consolidation() {
        let mut region_map = RegionMap::new();

        let free_region = MemoryRegion::new(
            PhyAddress::new(0x500).unwrap(),
            PhyAddress::new(0x1000).unwrap(),
            MemoryRegionType::Usable,
        );

        let reserved_region = MemoryRegion::new(
            PhyAddress::new(0x2000).unwrap(),
            PhyAddress::new(0x4000).unwrap(),
            MemoryRegionType::Reserved,
        );

        region_map.add_new_region(free_region).unwrap();
        region_map.add_new_region(reserved_region).unwrap();

        region_map.consolidate().unwrap();

        // TODO: We are not sure which region will be first, so maybe don't hard code this later
        let not_free_region = *region_map.regions.get(1).unwrap();
        let free_1_region = *region_map.regions.get(0).unwrap();

        assert_eq!(region_map.regions.len(), 2);

        assert_eq!(not_free_region, reserved_region);
        assert_eq!(free_1_region, free_region);
    }
}