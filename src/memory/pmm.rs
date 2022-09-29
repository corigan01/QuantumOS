/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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

use heapless::Vec;
use crate::bitset::BitSet;
use crate::error_utils::QuantumError;
use crate::memory::{PAGE_SIZE, PhysicalAddress, UsedMemoryKind};
use crate::memory::physical_memory::PhyRegion;
use crate::memory_utils::safe_ptr::SafePtr;
use crate::memory_utils::bool_vec::BoolVec;


struct PhyMemoryManagerComponent<'a> {
    phy_region: PhyRegion,
    buffer: BoolVec<'a>,
    used: usize,
}

impl<'a> PhyMemoryManagerComponent<'a> {
    pub fn new(region: PhyRegion, buffer: &'a mut [u8]) -> Self {
        Self {
            phy_region: region,
            buffer: BoolVec::new(buffer),
            used: 0
        }
    }

    pub fn allocate_page(&mut self) -> Option<PhysicalPageInformation> {
        if self.used < self.buffer.len() {
            let free_page = self.buffer.find_first_free()?;
            let starting_address_of_region = self.phy_region.start;
            let offset = free_page * PAGE_SIZE;

            let start = starting_address_of_region.as_u64() + offset as u64;
            let end = start + PAGE_SIZE as u64;

            return Some(PhysicalPageInformation {
                uid: free_page,
                start_address: PhysicalAddress::new(start),
                end_address: PhysicalAddress::new(end)
            });
        }

        None
    }

    pub fn free_page(&mut self, uid: usize) -> Result<(), QuantumError> {
        if uid < self.buffer.len() {
            self.buffer.set_bit(uid, false)?;

            Ok(())
        } else {
            Err(QuantumError::NoItem)
        }
    }



}

pub struct PhysicalPageInformation {
    pub uid: usize,
    pub start_address: PhysicalAddress,
    pub end_address: PhysicalAddress
}


pub struct PhyMemoryManager {
    components: Vec<PhyMemoryManagerComponent<'static>, 255>,
}

