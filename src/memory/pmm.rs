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

use core::mem::MaybeUninit;
use heapless::pool::{ Pool, singleton::Box };
use heapless::Vec;
use crate::{debug_println, debug_print };
use crate::bitset::BitSet;
use crate::memory::physical_memory::PhyRegionMap;
use crate::memory::{PhysicalAddress, UsedMemoryKind};

pub struct PhyMM {
    page_vector: Pool<PhySection>,
    pool_count: u16
}

impl PhyMM {
    pub fn new() -> Self {
        PhyMM {
            page_vector: Pool::new(),
            pool_count: 0
        }
    }

    fn find_free_pool_id(&self) -> Option<u64> {


        None
    }

    pub fn grow_pool(&self, bytes: &'static mut [u8]) {
        self.page_vector.grow(bytes);
    }

    pub fn commit_page(kind: UsedMemoryKind) {

    }


}

struct PhySection {
    page_vector: Vec<PhyPage, 255>,
    address_offset: PhysicalAddress,
}

pub fn test() {
    /*static mut INITIAL_PMM_MEMORY: [u8; 4096] = [0; 4096];

    let mut pool: Pool<PhySection> = Pool::new();

    unsafe { pool.grow(&mut INITIAL_PMM_MEMORY) };

    let vector = pool.alloc().unwrap();

    let init = vector.init(PhySection::new());*/
}


/// # Physical Page
/// A page is normally a 4k section of memory that is aligned to the next 4k section of memory.
/// This will allow us to calculate the address from a vector of address conversion stored in the
/// PhyMM. This makes this struct incredibly small and memory dense. We want to store all we can
/// in the smalled amount of memory because as total memory grows, the amount of pages does too.
#[derive(Debug, Clone, Copy)]
pub struct PhyPage(u8);

impl PhyPage {
    pub fn new() -> Self {
        PhyPage {
            0: 0
        }
    }

    pub fn set_used(&mut self, used: bool) {
        self.0.set_bit(7, used);
    }

    pub fn is_free(&self) -> bool {
        self.0.get_bit(7)
    }

    pub fn set_type(&mut self, kind: UsedMemoryKind) {
        self.0.set_bits(0..4, kind as u64);
    }

    pub fn get_type(&self) -> UsedMemoryKind {
        UsedMemoryKind::from_u8(self.0.get_bits(0..4))
    }

}

