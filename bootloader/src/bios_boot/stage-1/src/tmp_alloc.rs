/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
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

use core::alloc::{GlobalAlloc, Layout};
use core::mem::size_of;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::bios_println;
use crate::error::BootloaderError;


#[derive(Debug)]
pub struct AllocMemoryRegion {
    memory_location_ptr: *mut u8,
    memory_alloc_size: usize,
    used_space: usize,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MemoryAllocationEntry {
    magic: u64,
    allocation_size: u32,
    next_entry_ptr: u32,
}

unsafe impl Send for AllocMemoryRegion {}
unsafe impl Sync for AllocMemoryRegion {}

impl MemoryAllocationEntry {
    const MAGIC: u64 = 0xdeadbeefcafebab3;

    pub fn check_magic(&self) -> bool {
        self.magic == Self::MAGIC
    }

    pub fn generate_entry(entry_size: u32, next_entry: Option<u32>) -> Self {
        MemoryAllocationEntry {
            magic: Self::MAGIC,
            allocation_size: entry_size,
            next_entry_ptr: next_entry.unwrap_or(0)
        }
    }
}

lazy_static! {
    static ref TEMP_ALLOC: Mutex<Result<AllocMemoryRegion, BootloaderError>> = {
        Mutex::new(Err(BootloaderError::NoValid))
    };
}

impl AllocMemoryRegion {
    pub fn new(ptr: *mut u8, size: usize) -> Result<Self, BootloaderError> {
        if size < size_of::<MemoryAllocationEntry>() * 2 + 1 {
            return Err(BootloaderError::NotEnoughMemory)
        }


        let alloc_entry = Self {
            memory_location_ptr: ptr,
            memory_alloc_size: size,
            used_space: 0,
        };

        let memory_entry = MemoryAllocationEntry::generate_entry(0, None);
        alloc_entry.add_new_memory_alloc_entry(0, memory_entry);

        Ok(alloc_entry)
    }

    pub fn get_remaining_space(&self) -> usize {
        self.memory_alloc_size - self.used_space
    }

    fn add_new_memory_alloc_entry(&self, at: usize, entry: MemoryAllocationEntry) {
        let new_ptr = unsafe { self.memory_location_ptr.add(at) as *mut MemoryAllocationEntry };

        unsafe {
            *new_ptr = entry;
        }
    }

    fn iterate_through_array_and_return_on_true<F>(&self, function: F) -> Result<*mut u8, BootloaderError>
        where F: Fn(&MemoryAllocationEntry) -> bool {

        let mut working_ptr = self.memory_location_ptr;

        for i in 0..self.memory_alloc_size {
            let entry = unsafe {
                &*(working_ptr.add(i) as *const MemoryAllocationEntry)
            };

            if !entry.check_magic() {
                continue;
            }

            let result = function(entry);
        }


        todo!()
    }
}

unsafe impl GlobalAlloc for AllocMemoryRegion {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.iterate_through_array_and_return_on_true(
            |memory_entry| {
                bios_println!("{:#?}", memory_entry);

                false
            }
        ).unwrap();

        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!()
    }
}

pub fn populate_global_allocator(allocator_ptr: *mut u8, allocation_size: usize) -> Result<(), BootloaderError> {
    let mut binding = TEMP_ALLOC.lock();
    let mut temp_alloc = binding;
    let new_entry = AllocMemoryRegion::new(allocator_ptr, allocation_size)?;

    bios_println!("Set new allocator {:#?}", &new_entry);

    *temp_alloc = Ok(new_entry);

    Ok(())
}