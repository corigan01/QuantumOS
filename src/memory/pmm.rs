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
            self.buffer.set_bit(free_page, true).unwrap_or(());

            let starting_address_of_region = self.phy_region.start;
            let offset = free_page * PAGE_SIZE;

            let start = starting_address_of_region.as_u64() + offset as u64;
            let end = start + PAGE_SIZE as u64;

            return Some(PhysicalPageInformation {
                start_address: PhysicalAddress::new(start),
                end_address: PhysicalAddress::new(end)
            });
        }

        None
    }

    pub unsafe fn free_page(&mut self, uid: usize) -> Result<(), QuantumError> {
        if uid < self.buffer.len() {
            self.buffer.set_bit(uid, false)?;

            Ok(())
        } else {
            Err(QuantumError::NoItem)
        }
    }

    pub fn expand_buffer(&mut self, buffer: &'a mut [u8]) -> Result<(), QuantumError> {
        self.buffer.transfer_expand(buffer)
    }

    pub fn recommended_internal_buffer_size(&self) -> usize {
        let start = self.phy_region.start.as_u64();
        let end = self.phy_region.end.as_u64();

        let total_bytes = end - start;
        let total_pages = total_bytes / PAGE_SIZE as u64;

        (total_pages / 8) as usize
    }

    pub fn has_buf_storage_for_pages(&self) -> bool {
        let should_bytes = self.recommended_internal_buffer_size();
        let our_bytes = self.buffer.len();

        (should_bytes * 8) <= our_bytes
    }

}

pub struct PhysicalPageInformation {
    pub start_address: PhysicalAddress,
    pub end_address: PhysicalAddress
}

pub struct PhyMemoryManager<'a> {
    components: Vec<PhyMemoryManagerComponent<'a>, 20>,
}

impl<'a> PhyMemoryManager<'a> {
    pub fn new() -> Self {
        Self {
            components: Vec::new()
        }
    }

    pub fn recommended_bytes_to_store_allocation(region: PhyRegion) -> usize {
        let start = region.start.as_u64();
        let end = region.end.as_u64();

        let total_bytes = end - start;
        let total_pages = total_bytes / PAGE_SIZE as u64;
        let bytes = total_pages / 8;


        bytes as usize
    }

    pub fn insert_new_region(&mut self, region: PhyRegion, allocation: &'a mut [u8] ) -> Result<(), QuantumError> {
        let res = self.components.push(PhyMemoryManagerComponent::new(
            region,
            allocation
        ));

        if res.is_err() {
            Err(QuantumError::BufferFull)
        } else {
            Ok(())
        }
    }

    pub fn allocate(&mut self) -> Option<PhysicalPageInformation> {
        for mut i in 0..self.components.len() {
            let mut component = &mut self.components[i];

            if let Some(mut page) = component.allocate_page() {

                return Some(page);
            }
        }

        None
    }

    pub fn unallocate(&mut self, starting_address: PhysicalAddress) -> Result<(), QuantumError> {
        for mut i in 0..self.components.len() {
            let component = &mut self.components[i];
            let region_start = component.phy_region.start.as_u64();
            let region_end = component.phy_region.end.as_u64();

            if starting_address.as_u64() > region_start && starting_address.as_u64() < region_end {
                let page_id = (starting_address.as_u64() as usize) / PAGE_SIZE;

                unsafe { component.free_page(page_id) };
                return Ok(());
            }
        }

        Err(QuantumError::NoItem)
    }

}

#[cfg(test)]
mod test_case {
    use crate::memory::physical_memory::{PhyRegion, PhyRegionKind};
    use crate::memory::{PAGE_SIZE, PhysicalAddress};
    use crate::memory::pmm::{PhyMemoryManager, PhyMemoryManagerComponent};

    #[test_case]
    pub fn construct_a_component() {
        let mut region = PhyRegion::new();

        let dummy_size = 0x10000;
        let dummy_buffer_size = (dummy_size / PAGE_SIZE) / 8;

        // dummy region
        region.start = PhysicalAddress::new(0);
        region.end = PhysicalAddress::new(dummy_size as u64);
        region.kind = PhyRegionKind::Usable;

        let mut limited_lifetime_buffer = [0_u8; 196];

        let comp =
            PhyMemoryManagerComponent::new(region, &mut limited_lifetime_buffer);

        assert_eq!(comp.recommended_internal_buffer_size(), dummy_buffer_size);
        assert_eq!(comp.has_buf_storage_for_pages(), true);
    }

    #[test_case]
    pub fn try_allocating_and_freeing_pages() {
        let mut region = PhyRegion::new();

        let dummy_size = 1000 * PAGE_SIZE;
        let dummy_buffer_size = (dummy_size / PAGE_SIZE) / 8;

        // dummy region
        region.start = PhysicalAddress::new(0);
        region.end = PhysicalAddress::new(dummy_size as u64);
        region.kind = PhyRegionKind::Usable;

        let mut limited_lifetime_buffer = [0_u8; 196];

        let mut comp =
            PhyMemoryManagerComponent::new(region, &mut limited_lifetime_buffer);

        assert_eq!(comp.has_buf_storage_for_pages(), true);

        let alloc = comp.allocate_page().unwrap();

        assert_eq!(alloc.start_address.as_u64(), 0);
        assert_eq!(alloc.end_address.as_u64(), 0 + PAGE_SIZE as u64);

        // caller must ensure that the page is no longer used
        unsafe { comp.free_page(0) }.unwrap();
    }

    #[test_case]
    pub fn attempt_to_allocate_many_items() {
        let mut region = PhyRegion::new();

        let dummy_size = 1000 * PAGE_SIZE;
        let dummy_buffer_size = (dummy_size / PAGE_SIZE) / 8;

        // dummy region
        region.start = PhysicalAddress::new(0);
        region.end = PhysicalAddress::new(dummy_size as u64);
        region.kind = PhyRegionKind::Usable;

        let mut limited_lifetime_buffer = [0_u8; 196];

        let mut comp =
            PhyMemoryManagerComponent::new(region, &mut limited_lifetime_buffer);

        assert_eq!(comp.has_buf_storage_for_pages(), true);

        for i in 0_u64..100 {
            let alloc = comp.allocate_page().unwrap();

            assert_eq!(alloc.start_address.as_u64(), i * (PAGE_SIZE as u64));
            assert_eq!(alloc.end_address.as_u64(), i * (PAGE_SIZE as u64) + (PAGE_SIZE as u64));
        }

        // caller must ensure that the page is no longer used
        unsafe { comp.free_page(0) }.unwrap();
    }

    #[test_case]
    pub fn test_reallocating_buffer() {
        let mut region = PhyRegion::new();

        let dummy_size = 1000 * PAGE_SIZE;
        let dummy_buffer_size = (dummy_size / PAGE_SIZE) / 8;

        // dummy region
        region.start = PhysicalAddress::new(0);
        region.end = PhysicalAddress::new(dummy_size as u64);
        region.kind = PhyRegionKind::Usable;

        let mut limited_lifetime_buffer = [0_u8; 196];

        let mut comp =
            PhyMemoryManagerComponent::new(region, &mut limited_lifetime_buffer);

        assert_eq!(comp.has_buf_storage_for_pages(), true);

        let alloc = comp.allocate_page().unwrap();

        assert_eq!(alloc.start_address.as_u64(), 0);
        assert_eq!(alloc.end_address.as_u64(), 0 + PAGE_SIZE as u64);

        let mut second_limited_lifetime_buffer = [0_u8; 200];

        comp.expand_buffer(&mut second_limited_lifetime_buffer).unwrap();

        assert_eq!(comp.has_buf_storage_for_pages(), true);

        let page = comp.allocate_page().unwrap();

        assert_eq!(page.start_address.as_u64(), PAGE_SIZE as u64);
        assert_eq!(page.end_address.as_u64(), 2 * PAGE_SIZE as u64);
    }

    #[test_case]
    pub fn test_manager() {
        let mut limited_lifetime_buffer = [0_u8; 196];

        let mut pmm = PhyMemoryManager::new();
        let mut region = PhyRegion::new();

        let dummy_size = 1000 * PAGE_SIZE;
        let dummy_buffer_size = (dummy_size / PAGE_SIZE) / 8;

        // dummy region
        region.start = PhysicalAddress::new(0);
        region.end = PhysicalAddress::new(dummy_size as u64);
        region.kind = PhyRegionKind::Usable;
        
        pmm.insert_new_region(region, &mut limited_lifetime_buffer).unwrap();

        let allocation = pmm.allocate().unwrap();

        pmm.unallocate(allocation.start_address).unwrap();

        let re_allocation = pmm.allocate().unwrap();

        assert_eq!(allocation.start_address.as_u64(), re_allocation.start_address.as_u64());
    }


}
