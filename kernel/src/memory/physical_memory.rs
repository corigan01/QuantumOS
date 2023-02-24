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


use core::ops::Range;
use crate::memory::{PAGE_SIZE, PhysicalAddress};
use heapless::Vec;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PhyRegionKind {
    Usable,
    NotUsable
}

#[derive(Debug)]
pub struct PhyRegionMap {
    regions: [PhyRegion; 20],
    size: usize
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct PhyRegion {
    pub start: PhysicalAddress,
    pub end: PhysicalAddress,
    pub kind: PhyRegionKind
}

impl PhyRegion {
    pub fn new() -> Self {
        Self {
            start: PhysicalAddress::zero(),
            end: PhysicalAddress::zero(),
            kind: PhyRegionKind::NotUsable
        }
    }

    pub fn set_address_range(&mut self, range: Range<u64>) -> Self {
        self.start = PhysicalAddress::new(range.start as u64);
        self.end = PhysicalAddress::new(range.end as u64);

        self.clone()
    }

    pub fn set_type(&mut self, kind: PhyRegionKind) -> Self {
        self.kind = kind;

        self.clone()
    }

    pub fn get_start(&self) -> PhysicalAddress {
        self.start
    }

    pub fn get_size(&self) -> u64 {
        self.end.as_u64() - self.start.as_u64()
    }
}

impl PhyRegionMap {
    pub fn new() -> Self {
        Self {
            regions: [PhyRegion::new(); 20],
            size: 0
        }
    }

    pub fn add_entry(&mut self, entry: PhyRegion) {
        self.regions[self.size] = entry;

        self.size += 1;
    }

    pub fn do_regions_overlap(&self) -> bool {
        for i in 0..self.size {
            for e in (i + 1)..self.size {
                let checking_region = self.regions[i];
                let range_region = self.regions[e];

                if range_region.start <= checking_region.start && range_region.end >= checking_region.start {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_regions(&self, kind: PhyRegionKind) -> Option<Vec<PhyRegion, 20>> {
        let mut regions : Vec<PhyRegion, 20> = Vec::new();

        for i in 0..self.size {
            let region = self.regions[i];

            if region.kind == kind {

                // This should never panic as our vector is the same size as our region map.
                regions.push(region).unwrap();
            }
        }

        if !regions.is_empty() {
            return Some(regions);
        }

        None
    }

    pub fn get_total_bytes(&self, kind: PhyRegionKind) -> u64 {
        let free_regions = self.get_regions(kind);
         if let Some(some_free_regions) = free_regions {
            let mut memory_bytes : u64 = 0;

            for i in some_free_regions {
                memory_bytes += i.get_size();
            }

            memory_bytes
        } else { 0 }
    }

    pub fn get_usable_pages(&self) -> u64 {
        let bytes = self.get_total_bytes(PhyRegionKind::Usable);

        bytes / (PAGE_SIZE as u64)
    }
}