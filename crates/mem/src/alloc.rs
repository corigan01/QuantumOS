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

use core::{alloc::Layout, ptr::NonNull};

use crate::MemoryError;

extern crate alloc;

struct Buddy {
    free: bool,
    ptr: NonNull<Buddy>,
    len: usize,
    next: Option<NonNull<Buddy>>,
}

pub struct BootStrapAlloc {
    buddy: NonNull<Buddy>,
    len: usize,
}

impl BootStrapAlloc {
    pub fn new(memory_region: &'static mut [u8]) -> Self {
        assert!(memory_region.len() > size_of::<Buddy>());

        let mut buddy = NonNull::new(memory_region.as_mut_ptr().cast()).unwrap();

        unsafe {
            *buddy.as_mut() = Buddy {
                free: true,
                ptr: buddy,
                len: memory_region.len(),
                next: None,
            };
        }

        Self { buddy, len: 1 }
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
        }
        self.len -= 1;
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, MemoryError> {
        unsafe {
            let mut buddy = self.buddy;
            loop {
                let buddy_mut = buddy.as_mut();
                let min_len = layout.size()
                    + util::align_to(
                        (buddy_mut.ptr.addr().get() + size_of::<Buddy>()) as u64,
                        layout.align(),
                    ) as usize
                    + size_of::<Buddy>();

                if !buddy_mut.free || buddy_mut.len < min_len {
                    let Some(buddy_next) = buddy_mut.next else {
                        return Err(MemoryError::OutOfMemory);
                    };

                    buddy = buddy_next;
                    continue;
                }

                // Split
                if buddy_mut.len > min_len + (size_of::<Buddy>() * 2) {
                    let previous_next = buddy_mut.next;
                    let len = min_len
                        + util::align_to(
                            (buddy_mut.ptr.addr().get() + min_len) as u64,
                            align_of::<Buddy>(),
                        ) as usize;

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

                    return Ok(buddy_mut.ptr.as_ptr().byte_add(size_of::<Buddy>()).cast());
                } else {
                    buddy_mut.free = false;

                    return Ok(buddy_mut.ptr.as_ptr().byte_add(size_of::<Buddy>()).cast());
                }
            }
        }
    }

    pub unsafe fn free(&mut self, ptr: *mut u8, layout: Layout) -> Result<(), MemoryError> {
        let ptr_end = ptr as usize + layout.size();

        unsafe {
            let mut previous_buddy = self.buddy;
            loop {
                let Some(mut buddy) = previous_buddy.as_ref().next else {
                    let previous_mut = previous_buddy.as_mut();
                    let previous_ptr = previous_mut.ptr.addr().get() + size_of::<Buddy>();
                    let previous_end = previous_mut.ptr.addr().get() + previous_mut.len;

                    if previous_ptr < (ptr as usize) || previous_end < ptr_end {
                        return Err(MemoryError::NotFound);
                    }

                    previous_mut.free = true;
                    return Ok(());
                };

                let buddy_mut = buddy.as_mut();
                let buddy_ptr = buddy_mut.ptr.addr().get() + size_of::<Buddy>();
                let buddy_end = buddy_mut.ptr.addr().get() + buddy_mut.len;

                if buddy_ptr < (ptr as usize) || buddy_end < ptr_end {
                    continue;
                }

                buddy_mut.free = true;

                if let Some(next_buddy) = buddy_mut.next {
                    if next_buddy.as_ref().free {
                        self.melt_right(buddy);
                    }
                }

                if previous_buddy.as_ref().free {
                    self.melt_right(previous_buddy);
                }

                return Ok(());
            }
        }
    }
}
