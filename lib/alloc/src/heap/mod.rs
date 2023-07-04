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

use core::ptr;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::AllocErr;
use crate::heap::alloc::{KernelHeap, UnsafeAllocationObject};
use crate::memory_layout::MemoryLayout;

pub mod alloc;

pub trait AllocatorAPI {
    unsafe fn allocate(allocation_description: MemoryLayout) -> Result<UnsafeAllocationObject, AllocErr>;
    unsafe fn realloc<Type>(old_ptr: NonNull<Type>, new_alloc_desc: MemoryLayout) -> Result<UnsafeAllocationObject, AllocErr>;
    unsafe fn realloc_fill<Type>(old_ptr: NonNull<Type>, new_alloc_desc: MemoryLayout, fill: u8) -> Result<UnsafeAllocationObject, AllocErr>;
    unsafe fn free<Type>(ptr: NonNull<Type>) -> Result<(), AllocErr>;

    unsafe fn allocate_fill(allocation_description: MemoryLayout, fill: u8) -> Result<UnsafeAllocationObject, AllocErr> {
        let allocation = Self::allocate(allocation_description)?;
        ptr::write_bytes(allocation.ptr as *mut u8, fill, allocation.size);

        Ok(allocation)
    }

    unsafe fn allocate_zero(allocation_description: MemoryLayout) -> Result<UnsafeAllocationObject, AllocErr> {
        Self::allocate_fill(allocation_description, 0)
    }

    unsafe fn realloc_zero<Type>(old_ptr: NonNull<Type>, new_alloc_desc: MemoryLayout) -> Result<UnsafeAllocationObject, AllocErr> {
        Self::realloc_fill(old_ptr, new_alloc_desc, 0)
    }
}

pub struct GlobalAlloc;

impl AllocatorAPI for GlobalAlloc {
    unsafe fn allocate(allocation_description: MemoryLayout) -> Result<UnsafeAllocationObject, AllocErr> {
        reserve_lock();
        let u = get_global_alloc().allocate(allocation_description);
        free_lock();

        u
    }

    unsafe fn realloc<Type>(old_ptr: NonNull<Type>, new_alloc_desc: MemoryLayout) -> Result<UnsafeAllocationObject, AllocErr> {
        reserve_lock();
        let u = get_global_alloc().realloc(old_ptr, new_alloc_desc);
        free_lock();

        u
    }

    unsafe fn realloc_fill<Type>(old_ptr: NonNull<Type>, new_alloc_desc: MemoryLayout, fill: u8) -> Result<UnsafeAllocationObject, AllocErr> {
        reserve_lock();
        let u = get_global_alloc().realloc_fill(old_ptr, new_alloc_desc, fill);
        free_lock();

        u
    }

    unsafe fn free<Type>(ptr: NonNull<Type>) -> Result<(), AllocErr> {
        reserve_lock();
        let u = get_global_alloc().free(ptr);
        free_lock();

        u
    }
}

pub static mut THE_GLOBAL_ALLOC: Option<KernelHeap> = None;
pub static mut THE_GLOBAL_LOCK: AtomicBool = AtomicBool::new(false);

#[inline(never)]
pub fn reserve_lock() {
    unsafe {
        while !THE_GLOBAL_LOCK.compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire).is_ok() { }
    }
}

#[inline(never)]
pub fn free_lock() {
    unsafe {
        THE_GLOBAL_LOCK.store(false, Ordering::Release);
    }
}

pub fn set_global_alloc(alloc: KernelHeap) {
    unsafe {
        assert!(THE_GLOBAL_ALLOC.is_none(), "Can not move allocations! Unable to set the global allocator multiple times.");
        THE_GLOBAL_ALLOC = Some(alloc);
    }
}

pub fn get_global_alloc() -> &'static mut KernelHeap {
    unsafe {
        THE_GLOBAL_ALLOC.as_mut().expect("No global allocator set, please set a global allocator!")
    }
}

#[cfg(test)]
pub fn set_example_allocator(size_in_bytes: usize) {

    unsafe {
        if THE_GLOBAL_ALLOC.is_some() {
            return;
        }
    }

    reserve_lock();

    extern crate alloc;

    let memory_layout = core::alloc::Layout::from_size_align(size_in_bytes, 1).unwrap();
    unsafe {
        let allocation = alloc::alloc::alloc(memory_layout);

        let usable_region = crate::usable_region::UsableRegion::from_raw_parts(allocation, size_in_bytes).unwrap();
        let new_kern_heap = KernelHeap::new(usable_region).unwrap();

        set_global_alloc(new_kern_heap);
    }

    free_lock();
}