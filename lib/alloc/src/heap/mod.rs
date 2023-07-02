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

use core::ptr::NonNull;
use crate::AllocErr;
use crate::heap::alloc::{KernelHeap, UnsafeAllocationObject};
use crate::memory_layout::MemoryLayout;

pub mod alloc;

pub trait AllocatorAPI {
    unsafe fn allocate(allocation_description: MemoryLayout) -> Result<UnsafeAllocationObject, AllocErr>;
    unsafe fn realloc<Type>(old_ptr: NonNull<Type>, new_alloc_desc: MemoryLayout) -> Result<UnsafeAllocationObject, AllocErr>;
    unsafe fn free<Type>(ptr: NonNull<Type>) -> Result<(), AllocErr>;
}

pub struct GlobalAlloc;

impl AllocatorAPI for GlobalAlloc {
    unsafe fn allocate(allocation_description: MemoryLayout) -> Result<UnsafeAllocationObject, AllocErr> {
        get_global_alloc().allocate(allocation_description)
    }

    unsafe fn realloc<Type>(old_ptr: NonNull<Type>, new_alloc_desc: MemoryLayout) -> Result<UnsafeAllocationObject, AllocErr> {
        get_global_alloc().realloc(old_ptr, new_alloc_desc)
    }

    unsafe fn free<Type>(ptr: NonNull<Type>) -> Result<(), AllocErr> {
        get_global_alloc().free(ptr)
    }
}

static mut THE_GLOBAL_ALLOC: Option<KernelHeap> = None;
pub fn set_global_alloc(alloc: KernelHeap) {
    assert!(unsafe {THE_GLOBAL_ALLOC.is_none()});
    unsafe {
        THE_GLOBAL_ALLOC = Some(alloc)
    }
}

pub fn get_global_alloc() -> &'static mut KernelHeap {
    unsafe {
        THE_GLOBAL_ALLOC.as_mut().expect("No global allocator set, please set a global allocator!")
    }
}