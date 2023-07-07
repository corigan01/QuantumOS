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
use quantum_lib::address_utils::PAGE_SIZE;
use quantum_lib::address_utils::physical_address::PhyAddress;
use crate::pmm::PageAligned;


pub struct PhyPart {
    address: PageAligned,
    pages: usize,
    bitmap: Bitmap,
    many: Vec<(usize, usize)>
}

impl PhyPart {
    pub fn new(address: PageAligned, pages: usize) -> Self {
        Self {
            address,
            pages,
            bitmap: Bitmap::new(),
            many: Vec::new(),
        }
    }

    pub fn get_start_address(&self) -> PageAligned {
        self.address
    }

    pub fn get_qty_pages(&self) -> usize {
        self.pages
    }

    pub fn get_free_pages(&self) -> usize {
        let mut qty = 0;
        for i in 0..self.pages {
            if !self.bitmap.get_bit(i) {
                qty += 1;
            }
        }

        qty
    }

    #[inline]
    fn preform_address_calculation(&self, page_offset: usize) -> PageAligned {
        assert!(page_offset <= self.pages,
                "Can't get a page that is outside the range. Got page offset of {}, but only have {} pages available. ",
                page_offset,
                self.pages
        );

        let mut self_address = self.address.as_u64();
        self_address += (page_offset * PAGE_SIZE) as u64;

        PhyAddress::new(self_address).unwrap().try_aligned().unwrap()
    }

    #[inline]
    fn preform_reverse_address_calculation(&self, address: PageAligned) -> usize {
        assert!(address >= self.address,
                "Expected given address to be larger then base address. Given {}, base {}. ",
                address.as_u64(),
                self.address.as_u64(),
        );

        let norm_address = address.as_u64() - self.address.as_u64();
        let bit_index = norm_address / (PAGE_SIZE as u64);

        assert!(bit_index <= self.pages as u64,
                "Expected an address within the range of available pages. Got {} (P_OFF={}), but expected it to be lower then {} (P_OFF={}).",
                address.as_u64(),
                bit_index,
                self.preform_address_calculation(self.pages).as_u64(),
                self.pages
        );

        bit_index as usize
    }

    pub fn first_of(&self, flag: bool) -> Option<PageAligned> {
        let first = self.bitmap.first_of(flag)?;

        Some(self.preform_address_calculation(first))
    }

    pub fn first_of_many(&self, flag: bool, qty: usize) -> Option<PageAligned> {
        let first_of_many = self.bitmap.first_of_many(flag, qty)?;

        Some(self.preform_address_calculation(first_of_many))
    }

    pub fn reserve_first_free(&mut self) -> Option<PageAligned> {
        let first = self.bitmap.first_of(false)?;

        // Since bitmap is 'infinite', it will always find a free bit. This means
        // we must make sure that this free bit is within our range that we have.
        if first > self.pages {
            return None;
        }

        self.bitmap.set_bit(first, true);

        Some(self.preform_address_calculation(first))
    }

    pub fn reserve_first_free_of_many(&mut self, qty: usize) -> Option<PageAligned> {
        let first_of_many = self.bitmap.first_of_many(false, qty)?;

        if first_of_many > self.pages {
            return None;
        }

        self.bitmap.set_many(first_of_many, true, qty);

        self.many.push((first_of_many, qty));

        Some(self.preform_address_calculation(first_of_many))
    }

    pub fn free(&mut self, address: PageAligned) {
        let bit_index = self.preform_reverse_address_calculation(address);

        let many_qty = self.many.iter()
            .enumerate()
            .find_map(|(index, (first, qty))| {
           if bit_index == *first {
               Some((index, qty))
           } else {
               None
           }
        });

        if let Some((index, many_qty)) = many_qty {
            self.bitmap.set_many(bit_index, false, *many_qty);
            self.many.remove(index);
        }

        self.bitmap.set_bit(bit_index, false);
    }
}

#[cfg(test)]
mod test {
    use crate::set_example_allocator;
    use super::*;

    #[test]
    fn test_creating_new_phy_part() {
        set_example_allocator(4096);

        let start_address = (PAGE_SIZE * 2) as u64;
        let qty_pages = 10;

        let phy_part = PhyPart::new(
            PhyAddress::new(start_address).unwrap().try_aligned().unwrap(),
            qty_pages
        );

        assert_eq!(phy_part.first_of(false).unwrap().as_u64(), start_address);
    }

    #[test]
    fn test_allocating() {
        set_example_allocator(4096);

        let start_address = (PAGE_SIZE * 2) as u64;
        let size = 100;

        let mut phy_part = PhyPart::new(
            PhyAddress::new(start_address).unwrap().try_aligned().unwrap(),
            size
        );

        let first_free = phy_part.reserve_first_free().unwrap();

        assert_eq!(first_free.as_u64(), start_address);

        let second_free = phy_part.reserve_first_free().unwrap();

        assert_eq!(second_free.as_u64(), start_address + (PAGE_SIZE as u64));
    }
}