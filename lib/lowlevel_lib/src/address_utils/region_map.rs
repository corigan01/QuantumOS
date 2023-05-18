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

use crate::address_utils::addressable::Addressable;
use crate::address_utils::region::{MemoryRegion, MemoryRegionType};
use crate::bytes::Bytes;
use crate::heapless_vector::{HeaplessVec, HeaplessVecErr};

pub struct RegionMap<Type> {
    regions: HeaplessVec<MemoryRegion<Type>, 20>
}

impl<Type> RegionMap<Type>
    where Type: Addressable + Copy {
    pub fn new() -> Self {
        Self {
            regions: Default::default()
        }
    }
    
    pub fn condense_gaps(&mut self) {
        
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