/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

use crate::MemoryError;
use core::{
    alloc::{GlobalAlloc, Layout},
    fmt::Debug,
    ptr::NonNull,
};
use lldebug::sync::Mutex;
use util::{align_to, is_align_to};

struct Buddy {
    free: bool,
    ptr: NonNull<Buddy>,
    len: usize,
    next: Option<NonNull<Buddy>>,
}

pub struct BootStrapAlloc {
    buddy: NonNull<Buddy>,
    len: usize,
    bytes_used: usize,
    bytes_total: usize,
}

unsafe impl Send for BootStrapAlloc {}
unsafe impl Sync for BootStrapAlloc {}

impl BootStrapAlloc {
    pub fn new(memory_region: &mut [u8]) -> Self {
        assert!(memory_region.len() > size_of::<Buddy>() * 2);

        let buddy = NonNull::new(memory_region.as_mut_ptr().cast()).unwrap();
        let alignment = buddy.align_offset(align_of::<Buddy>());
        let mut aligned_buddy = unsafe { buddy.byte_add(alignment) };

        unsafe {
            *aligned_buddy.as_mut() = Buddy {
                free: true,
                ptr: aligned_buddy,
                len: memory_region.len() - alignment,
                next: None,
            };
        }

        Self {
            buddy: aligned_buddy,
            len: 1,
            bytes_used: alignment + size_of::<Buddy>(),
            bytes_total: memory_region.len(),
        }
    }

    unsafe fn melt_right(&mut self, mut buddy: NonNull<Buddy>) {
        unsafe {
            let current = buddy.as_mut();
            let Some(mut next) = current.next else {
                return;
            };

            let next = next.as_mut();
            current.next = next.next;
            current.len += next.len;

            if next.free {
                self.bytes_used -= size_of::<Buddy>();
            } else {
                self.bytes_used -= next.len;
            }
        }
        self.len -= 1;
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, MemoryError> {
        unsafe {
            let mut buddy = self.buddy;
            loop {
                let buddy_mut = buddy.as_mut();
                let align_len = align_to(buddy.add(1).addr().get() as u64, layout.align()) as usize
                    - buddy.add(1).addr().get();
                let min_len = layout.size() + align_len + size_of::<Buddy>();

                if !buddy_mut.free || buddy_mut.len < min_len {
                    let Some(buddy_next) = buddy_mut.next else {
                        return Err(MemoryError::OutOfAllocMemory);
                    };

                    buddy = buddy_next;
                    continue;
                }

                // Split
                let ret_ptr = if buddy_mut.len > min_len + (size_of::<Buddy>() * 2) {
                    let previous_next = buddy_mut.next;
                    let len = align_to((buddy.addr().get() + min_len) as u64, align_of::<Buddy>())
                        as usize
                        - buddy.addr().get();

                    let next_buddy = buddy.byte_add(len);
                    *next_buddy.as_ptr() = Buddy {
                        free: true,
                        ptr: next_buddy,
                        len: buddy_mut.len - len,
                        next: previous_next,
                    };

                    buddy_mut.free = false;
                    buddy_mut.len = len;
                    buddy_mut.next = Some(next_buddy);
                    self.len += 1;
                    self.bytes_used += buddy_mut.len;

                    buddy_mut
                        .ptr
                        .as_ptr()
                        .byte_add(size_of::<Buddy>() + align_len)
                        .cast()
                } else {
                    buddy_mut.free = false;
                    self.bytes_used += buddy_mut.len - size_of::<Buddy>();

                    buddy_mut
                        .ptr
                        .as_ptr()
                        .byte_add(size_of::<Buddy>() + align_len)
                        .cast()
                };

                assert!(
                    is_align_to(ret_ptr as u64, layout.align()),
                    "Alloc was about to return unaligned PTR"
                );

                return Ok(ret_ptr);
            }
        }
    }

    pub unsafe fn free(&mut self, ptr: *mut u8, layout: Layout) -> Result<(), MemoryError> {
        let ptr_end = ptr as usize + layout.size();

        unsafe {
            let mut previous_buddy = self.buddy;
            let mut index = 0;

            loop {
                let previous_mut = previous_buddy.as_mut();
                let previous_ptr = previous_mut.ptr.addr().get() + size_of::<Buddy>();
                let previous_end = previous_mut.ptr.addr().get() + previous_mut.len;

                let Some(mut buddy) = previous_buddy.as_ref().next else {
                    if previous_ptr < (ptr as usize) || previous_end < ptr_end {
                        return Err(MemoryError::NotFound);
                    }

                    previous_mut.free = true;
                    self.bytes_used -= previous_mut.len - size_of::<Buddy>();
                    return Ok(());
                };

                let buddy_mut = buddy.as_mut();
                let buddy_ptr = buddy_mut.ptr.addr().get() + size_of::<Buddy>();
                let buddy_end = buddy_mut.ptr.addr().get() + buddy_mut.len;

                if index == 0 && previous_ptr >= (ptr as usize) && previous_end >= ptr_end {
                    previous_mut.free = true;

                    if buddy_mut.free {
                        self.melt_right(previous_buddy);
                    }

                    return Ok(());
                }

                if buddy_ptr < (ptr as usize) || buddy_end < ptr_end {
                    previous_buddy = buddy;
                    index += 1;
                    continue;
                }

                buddy_mut.free = true;

                if let Some(next_buddy) = buddy_mut.next {
                    if next_buddy.as_ref().free {
                        self.melt_right(buddy);
                    }
                }

                // if previous_buddy.as_ref().free {
                //     self.melt_right(previous_buddy);
                // } else {
                //     self.bytes_used -= buddy_mut.len - size_of::<Buddy>();
                // }

                return Ok(());
            }
        }
    }
}

impl Debug for BootStrapAlloc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootStrapAlloc")
            .field("len", &self.len)
            .field("bytes_used", &self.bytes_used)
            .field("bytes_free", &(self.bytes_total - self.bytes_used))
            .field("bytes_total", &self.bytes_total)
            .field("buddy", unsafe { self.buddy.as_ref() })
            .finish()
    }
}

impl Debug for Buddy {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[allow(unused)]
        #[derive(Debug)]
        struct BuddyItem {
            free: bool,
            ptr: NonNull<Buddy>,
            len: usize,
            next: Option<NonNull<Buddy>>,
        }

        let mut list = f.debug_list();
        unsafe {
            let mut buddy = self.ptr;
            loop {
                let buddy_ref = buddy.as_ref();

                list.entry(&BuddyItem {
                    free: buddy_ref.free,
                    ptr: buddy_ref.ptr,
                    len: buddy_ref.len,
                    next: buddy_ref.next,
                });

                let Some(next_buddy) = buddy_ref.next else {
                    break;
                };

                buddy = next_buddy;
            }
        }

        list.finish()
    }
}

#[derive(Debug)]
struct InnerAllocator {
    init_alloc: Option<BootStrapAlloc>,
}

impl InnerAllocator {
    pub const fn new() -> Self {
        Self { init_alloc: None }
    }
}

static INNER_ALLOC: Mutex<InnerAllocator> = Mutex::new(InnerAllocator::new());

/// Give bytes to the init alloc.
pub fn provide_init_region(region: &'static mut [u8]) {
    let mut inner = INNER_ALLOC.lock();
    lldebug::logln!("Kernel init heap ({} Bytes)", region.len());
    inner.init_alloc = Some(BootStrapAlloc::new(region));
}

pub fn dump_allocator() {
    let inner = INNER_ALLOC.lock();
    lldebug::logln!("{:#?}", inner);
}

pub struct KernelAllocator {}

impl KernelAllocator {
    pub const fn new() -> Self {
        Self {}
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut inner = INNER_ALLOC.lock();

        unsafe { inner.init_alloc.as_mut().unwrap().alloc(layout).unwrap() }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut inner = INNER_ALLOC.lock();

        unsafe {
            inner
                .init_alloc
                .as_mut()
                .unwrap()
                .free(ptr, layout)
                .unwrap()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use core::alloc::Layout;
    extern crate std;

    #[test]
    fn test_buddy_new() {
        lldebug::testing_stdout!();
        let len = 10 * util::consts::KIB;
        let mem_region = unsafe {
            core::slice::from_raw_parts_mut(
                std::alloc::alloc_zeroed(Layout::from_size_align(len, 1).unwrap()),
                len,
            )
        };

        let mut ptrs = std::vec::Vec::new();
        let mut alloc = BootStrapAlloc::new(mem_region);

        for i in 0..10 {
            let ptr = unsafe { alloc.alloc(Layout::new::<u8>()).unwrap() };
            unsafe { *ptr = i };
            assert_eq!(unsafe { *ptr }, i);
            ptrs.push(ptr);
        }

        for i in 0..10 {
            let ptr = ptrs[i as usize];
            assert_eq!(unsafe { *ptr }, i);
            unsafe { alloc.free(ptr, Layout::new::<u8>()).unwrap() };
        }
    }
}
