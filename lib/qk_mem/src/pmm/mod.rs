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

use qk_alloc::bitmap::Bitmap;
use qk_alloc::vec::Vec;
use quantum_lib::address_utils::physical_address::{Aligned, PhyAddress};
use quantum_lib::address_utils::region::{MemoryRegion, MemoryRegionType};

pub enum PhyAllocErr {
    NotAligned,
    NotFree
}

pub struct PhyPage {
    address: PhyAddress<Aligned, 12>
}

pub struct PhysicalAllocator {
    memory_bitmap: Vec<(MemoryRegion<PhyAddress<Aligned, 12>>, Bitmap)>
}

impl PhysicalAllocator {
    pub const fn new() -> Self {
        Self {
            memory_bitmap: Vec::new()
        }
    }

    pub fn add_region(&mut self, region: MemoryRegion<PhyAddress>) -> Result<(), PhyAllocErr> {
        if region.region_type() != MemoryRegionType::Usable {
            return Err(PhyAllocErr::NotFree);
        }

        let Ok(new_aligned_start_address) = region.get_end_address().try_aligned() else {
            return Err(PhyAllocErr::NotAligned);
        };

        let new_memory_region = MemoryRegion::new(
            new_aligned_start_address,
            region.get_end_address().strip_unaligned_bits_to_align_address(),
            MemoryRegionType::Usable
        );

        self.memory_bitmap.push((new_memory_region, Bitmap::new()));

        Ok(())
    }

    pub fn reserve_page(&mut self) -> Result<PhyPage, PhyAllocErr> {
        self.memory_bitmap.iter().map(|region| &region.1).map(|bitmap| r.iter())

        todo!()
    }


}