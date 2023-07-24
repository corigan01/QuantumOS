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

use qk_alloc::vec::Vec;
use quantum_lib::address_utils::PAGE_SIZE;
use quantum_lib::address_utils::physical_address::{Aligned, PhyAddress};
use quantum_lib::address_utils::region::MemoryRegion;
use quantum_lib::address_utils::region_map::RegionMap;
use crate::pmm::phy_part::PhyPart;

pub mod phy_part;

pub type PageAligned = PhyAddress<Aligned, 12>;

pub enum PhyAllocErr {
    NotAligned,
    NotFree,
    NotEnoughMemory
}

#[allow(dead_code)]
pub struct PhysicalMemoryManager {
    usable: Vec<PhyPart>,
    kernel: Vec<MemoryRegion<PhyAddress>>,
    io: Vec<MemoryRegion<PhyAddress>>,
    other: Vec<MemoryRegion<PhyAddress>>
}

impl PhysicalMemoryManager {
    pub fn new(map: &RegionMap<PhyAddress>) -> Self {
        let mut free_allocations: Vec<PhyPart> = map.iter()
            .filter(|region| region.is_usable() && region.size() > PAGE_SIZE as u64)
            .map(|region| {
                PhyPart::new(
                    region.get_start_address().align_up(),

                    // We have to subtract one because aligning up will mean that we lose the bottom one page
                    (region.size() as usize / PAGE_SIZE) - 1
                )
            })
            .collect();

        free_allocations.sort_by(|this, other| {
            let this = this.get_start_address().as_u64();
            let other = other.get_start_address().as_u64();

            this.cmp(&other)
        });

        let kernel_allocations: Vec<MemoryRegion<PhyAddress>> = map.iter()
            .filter(|region| region.is_kernel())
            .collect();

        let other: Vec<MemoryRegion<PhyAddress>> = map.iter()
            .filter(|region| region.is_reserved())
            .collect();

        Self {
            usable: free_allocations,
            kernel: kernel_allocations,
            io: Vec::new(),
            other
        }
    }

    pub fn allocate_free_page(&mut self) -> Result<PageAligned, PhyAllocErr> {
        let Some(address) = self.usable.iter_mut().find_map(|entry| {
            entry.reserve_first_free()
        }) else {
            return Err(PhyAllocErr::NotEnoughMemory);
        };

        Ok(address)
    }

    pub fn allocate_free_pages(&mut self, qty: usize) -> Result<PageAligned, PhyAllocErr> {
        let Some(start_address) = self.usable.iter_mut().find_map(|entry| {
            entry.reserve_first_free_of_many(qty)
        }) else {
            return Err(PhyAllocErr::NotEnoughMemory);
        };

        Ok(start_address)
    }


}