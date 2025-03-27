/*
  ____                 __               __  __
 / __ \__ _____ ____  / /___ ____ _    / / / /__ ___ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /_/ (_-</ -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/  \____/___/\__/_/
  Part of the Quantum OS Kernel

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

use crate::sync::Mutex;
use core::{
    alloc::{GlobalAlloc, Layout},
    fmt::{Debug, Display},
    ptr::{NonNull, null_mut},
    sync::atomic::{AtomicPtr, Ordering},
};
use quantum_portal::{
    MapMemoryError, MemoryLocation, MemoryProtections,
    sys_client::{map_memory, unmap_memory},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryAllocationError {
    OutOfMemory,
    OutOfSystemMemory,
    DoubleFree,
    NotAllocated,
    NotInRegion,
}

impl Display for MemoryAllocationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl core::error::Error for MemoryAllocationError {}

impl From<MapMemoryError> for MemoryAllocationError {
    fn from(value: MapMemoryError) -> Self {
        match value {
            MapMemoryError::InvalidLength(l) => {
                panic!("Inner alloc error: Invalid allocation length = {l}")
            }
            MapMemoryError::MappingMemoryError => {
                panic!("Inner alloc error: OS mapping memory error!")
            }
            MapMemoryError::OutOfMemory => Self::OutOfSystemMemory,
        }
    }
}

pub type Result<T> = core::result::Result<T, MemoryAllocationError>;

#[derive(Debug, PartialEq, Eq)]
enum BuddyState {
    Free,
    Used { layout: Layout },
}

#[derive(Debug)]
struct BuddyNode {
    next: Option<NonNull<BuddyNode>>,
    prev: Option<NonNull<BuddyNode>>,
    state: BuddyState,
    size: usize,
}

struct BuddyAllocator {
    head: Option<NonNull<BuddyNode>>,
    region_start: NonNull<u8>,
    region_end: NonNull<u8>,
}

impl BuddyAllocator {
    const fn new(ptr: NonNull<u8>, len: usize) -> Self {
        let buddy_allocator = Self {
            head: None,
            region_start: ptr,
            region_end: unsafe { ptr.byte_add(len) },
        };
        buddy_allocator
    }

    fn head(&mut self) -> NonNull<BuddyNode> {
        let buddy = *self.head.get_or_insert_with(|| {
            let region = self.region_start..self.region_end;
            let offset = self.region_start.align_offset(align_of::<BuddyNode>());
            let new_buddy = unsafe { self.region_start.byte_add(offset) }.cast::<BuddyNode>();

            assert!(region.contains(&new_buddy.cast::<u8>()));

            unsafe {
                new_buddy.write(BuddyNode {
                    next: None,
                    prev: None,
                    state: BuddyState::Free,
                    size: (self.region_end.addr().get() - new_buddy.addr().get())
                        - size_of::<BuddyNode>(),
                });
            }

            new_buddy
        });

        self.safety_check_buddy(buddy);
        buddy
    }

    /// Asserts that the given buddy is within the allocation region provided, and that it is properly formed.
    #[inline]
    fn safety_check_buddy(&self, buddy: NonNull<BuddyNode>) -> BuddyNode {
        let region = self.region_start..self.region_end;

        debug_assert!(buddy.is_aligned());
        assert!(region.contains(&buddy.cast::<u8>()));

        let buddy_read = unsafe { buddy.read() };
        debug_assert!(
            buddy_read
                .next
                .is_none_or(|next| { region.contains(&next.cast::<u8>()) })
        );
        debug_assert!(
            buddy_read
                .prev
                .is_none_or(|prev| { region.contains(&prev.cast::<u8>()) })
        );
        debug_assert_ne!(buddy_read.size, 0, "{:#?}", self);

        buddy_read
    }

    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8> {
        if layout.size() == 0 {
            return Ok(self.region_start.as_ptr());
        }

        let mut cursor = self.head();

        loop {
            let cursor_read = self.safety_check_buddy(cursor);
            if matches!(cursor_read.state, BuddyState::Used { .. }) {
                cursor = cursor_read.next.ok_or(MemoryAllocationError::OutOfMemory)?;
                continue;
            }

            let post_header_ptr = unsafe { cursor.byte_add(size_of::<BuddyNode>()) };
            let post_header_size = cursor_read.size;
            let end_region_ptr = unsafe { post_header_ptr.byte_add(post_header_size) };

            let type_alignment_cost = post_header_ptr.cast::<u8>().align_offset(layout.align());
            let type_size = type_alignment_cost + layout.size();

            // Check if this buddy can fit the allocation
            if post_header_size < type_size {
                if let Some(next_cursor) = cursor_read.next {
                    cursor = next_cursor;
                } else {
                    return Err(MemoryAllocationError::OutOfMemory);
                }
                continue;
            }

            let post_allocation_bytes = post_header_size - type_size;
            let next_header_alignmnet_cost = unsafe {
                post_header_ptr
                    .cast::<u8>()
                    .byte_add(type_size)
                    .align_offset(align_of::<BuddyNode>())
            };

            // If we can fit another allocation buddy in this region
            if post_allocation_bytes > next_header_alignmnet_cost + (2 * size_of::<BuddyNode>()) {
                let mut next_buddy_ptr =
                    unsafe { post_header_ptr.byte_add(type_size + next_header_alignmnet_cost) };

                debug_assert!(next_buddy_ptr.is_aligned());
                debug_assert!(
                    unsafe { next_buddy_ptr.byte_add(size_of::<BuddyNode>()) } < end_region_ptr
                );

                let new_post_header_size =
                    next_buddy_ptr.addr().get() - post_header_ptr.addr().get();
                let next_size = (end_region_ptr.addr().get() - next_buddy_ptr.addr().get())
                    - size_of::<BuddyNode>();

                // Resolve new node's `next` and `prev` connections
                unsafe {
                    let next_mut = next_buddy_ptr.as_mut();

                    next_mut.prev = Some(cursor);
                    next_mut.size = next_size;
                    next_mut.state = BuddyState::Free;

                    if let Some(mut next) = cursor_read.next {
                        next_mut.next = Some(next);
                        next.as_mut().prev = Some(next_buddy_ptr);
                    } else {
                        next_mut.next = None;
                    }

                    let cursor_mut = cursor.as_mut();
                    cursor_mut.next = Some(next_buddy_ptr);
                    cursor_mut.size = new_post_header_size;
                };
            }

            // Update buddy's status
            unsafe {
                let cursor_mut = cursor.as_mut();
                cursor_mut.state = BuddyState::Used { layout };
            }

            let ret_ptr: *mut u8 = unsafe { post_header_ptr.byte_add(type_alignment_cost) }
                .cast()
                .as_ptr();

            debug_assert_eq!(ret_ptr.addr() as u64 & (layout.align() as u64 - 1), 0);
            unsafe { ret_ptr.write_bytes(0, layout.size()) };

            return Ok(ret_ptr);
        }
    }

    fn combine(&mut self, cursor: NonNull<BuddyNode>) {
        // Combine Left
        let mut current = cursor;
        loop {
            let current_read = self.safety_check_buddy(current);
            let Some(prev) = current_read.prev else {
                break;
            };
            let prev_read = self.safety_check_buddy(prev);

            if !matches!(prev_read.state, BuddyState::Free)
                || !matches!(current_read.state, BuddyState::Free)
            {
                break;
            }

            unsafe {
                prev.write(BuddyNode {
                    next: current_read.next,
                    prev: prev_read.prev,
                    state: BuddyState::Free,
                    size: current_read.size + prev_read.size + size_of::<BuddyNode>(),
                });

                if let Some(mut next) = current_read.next {
                    next.as_mut().prev = Some(prev);
                }
            }

            current = prev;
        }

        // Combine Right
        loop {
            let current_read = self.safety_check_buddy(current);
            let Some(next) = current_read.next else {
                break;
            };
            let next_read = self.safety_check_buddy(next);

            if !matches!(next_read.state, BuddyState::Free)
                || !matches!(current_read.state, BuddyState::Free)
            {
                break;
            }

            unsafe {
                current.write(BuddyNode {
                    next: next_read.next,
                    prev: current_read.prev,
                    state: BuddyState::Free,
                    size: current_read.size + next_read.size + size_of::<BuddyNode>(),
                });

                if let Some(mut next_next) = next_read.next {
                    next_next.as_mut().prev = Some(current);
                }
            }

            current = next;
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) -> Result<()> {
        if self.region_end.as_ptr() < ptr && self.region_start.as_ptr() > ptr {
            return Err(MemoryAllocationError::NotInRegion);
        }

        if layout.size() == 0 {
            assert_eq!(ptr, self.region_start.as_ptr());
        }

        let mut cursor = self.head();

        loop {
            let cursor_read = self.safety_check_buddy(cursor);
            let post_header_size = cursor_read.size;
            let post_header_ptr = unsafe { cursor.byte_add(size_of::<BuddyNode>()) }.cast::<u8>();
            let post_header_end =
                unsafe { post_header_ptr.byte_add(post_header_size) }.cast::<u8>();

            if !(post_header_ptr.as_ptr()..post_header_end.as_ptr()).contains(&ptr) {
                cursor = cursor_read
                    .next
                    .expect("reached end of region, but didn't find ptr to free!");
                continue;
            }

            // check that this region is valid
            match cursor_read.state {
                BuddyState::Free => return Err(MemoryAllocationError::DoubleFree),
                BuddyState::Used {
                    layout: state_layout,
                } if state_layout != layout => {
                    return Err(MemoryAllocationError::NotAllocated);
                }
                _ => (),
            }

            unsafe { cursor.as_mut().state = BuddyState::Free };
            self.combine(cursor);

            break Ok(());
        }
    }
}

impl Debug for BuddyAllocator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct Fields {
            head_ptr: Option<NonNull<BuddyNode>>,
        }

        impl Debug for Fields {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let mut list = f.debug_list();

                if let Some(mut alloc) = self.head_ptr {
                    list.entry(&unsafe { alloc.read() });

                    while let Some(next) = unsafe { alloc.read().next } {
                        list.entry(&unsafe { next.read() });
                        alloc = next;
                    }
                }

                list.finish()
            }
        }

        f.debug_struct(stringify!(BuddyAllocator))
            .field("head", &self.head)
            .field(
                "region_len",
                &(self.region_end.addr().get() - self.region_start.addr().get()),
            )
            .field(
                "alloc",
                &Fields {
                    head_ptr: self.head,
                },
            )
            .finish()
    }
}

/// A container for one region `memory_map`'ed with its own allocator
struct MemoryMapRegion {
    alloc: Mutex<Option<BuddyAllocator>>,
    next: AtomicPtr<MemoryMapRegion>,
}

impl MemoryMapRegion {
    const REGION_SIZE: usize = 1024 * 1024;

    /// Create a new memory region
    pub const fn new() -> Self {
        Self {
            alloc: Mutex::new(None),
            next: AtomicPtr::new(null_mut()),
        }
    }

    fn new_buddy() -> Result<BuddyAllocator> {
        let memory_region_ptr = map_memory(
            MemoryLocation::Anywhere,
            MemoryProtections::ReadWrite,
            Self::REGION_SIZE,
        )?;
        Ok(BuddyAllocator::new(
            NonNull::new(memory_region_ptr).expect("Mapping memory should never return 0"),
            Self::REGION_SIZE,
        ))
    }

    pub unsafe fn alloc(&self, layout: Layout) -> Result<*mut u8> {
        let mut alloc_lock = self.alloc.lock();

        // Try to create the region if one doesn't exist
        if alloc_lock.is_none() {
            if let Some(previous_alloc) = alloc_lock.replace(Self::new_buddy()?) {
                unreachable!(
                    "Tried to replace an existing MemoryMapRegion -- {:#?}",
                    previous_alloc
                );
            }
        }

        let alloc_inner = alloc_lock
            .as_mut()
            .expect("Just allocated a region, cannot be none!");

        unsafe {
            match alloc_inner.alloc(layout) {
                Ok(allocated) => Ok(allocated),
                Err(MemoryAllocationError::OutOfMemory) => {
                    // Only try to exchange if the ptr is first found to be null
                    //
                    // This saves a lot of work because its rare that two threads will
                    // both try to allocate a new region at the same time.
                    let mut next_ptr = self.next.load(Ordering::Relaxed);
                    if next_ptr.is_null() {
                        let mut new_buddy = Self::new_buddy()?;
                        let new_region_start = new_buddy.region_start;

                        // Use the new allocator to allocate itself
                        next_ptr = new_buddy
                            .alloc(Layout::new::<MemoryMapRegion>())?
                            .cast::<MemoryMapRegion>();

                        *next_ptr = MemoryMapRegion {
                            alloc: Mutex::new(Some(new_buddy)),
                            next: AtomicPtr::new(null_mut()),
                        };

                        while let Err(failed_set) = self.next.compare_exchange(
                            null_mut(),
                            next_ptr,
                            Ordering::SeqCst,
                            Ordering::Relaxed,
                        ) {
                            // If the previous ptr is null, try to exchange again
                            if failed_set.is_null() {
                                continue;
                            }

                            // If we failed the exchange race, we drop our new region and use the
                            // failed one
                            unmap_memory(new_region_start.as_ptr());
                            next_ptr = failed_set;
                        }
                    }

                    (&mut *next_ptr).alloc(layout)
                }
                Err(other_error) => Err(other_error),
            }
        }
    }

    pub unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) -> Result<()> {
        let mut alloc_lock = self.alloc.lock();

        let Some(ref mut inner) = *alloc_lock else {
            return Err(MemoryAllocationError::NotAllocated);
        };

        unsafe {
            match inner.dealloc(ptr, layout) {
                Ok(_) => Ok(()),
                Err(MemoryAllocationError::NotInRegion) => {
                    let next_ptr = self.next.load(Ordering::Relaxed);

                    if next_ptr.is_null() {
                        return Err(MemoryAllocationError::NotAllocated);
                    }

                    (&*next_ptr).dealloc(ptr, layout)
                }
                Err(err) => Err(err),
            }
        }
    }
}

impl Drop for MemoryMapRegion {
    fn drop(&mut self) {
        let next_ptr = self.next.load(Ordering::Relaxed);

        if !next_ptr.is_null() {
            unsafe {
                core::ptr::drop_in_place(next_ptr);
            }
        }

        let alloc_lock = self.alloc.lock();
        let Some(ref alloc) = *alloc_lock else {
            return;
        };

        // Remove the inner allocator region
        let ptr = alloc.region_start.as_ptr();
        unmap_memory(ptr);
    }
}

/// Heap allocated region filled with memory mappings
pub struct QuantumHeap {
    head_region: MemoryMapRegion,
}

impl QuantumHeap {
    /// Lazy init the allocator
    pub const fn new() -> Self {
        Self {
            head_region: MemoryMapRegion::new(),
        }
    }

    /// Allocate memory with the allocator
    #[inline]
    pub unsafe fn inner_allocate(&self, layout: Layout) -> Result<*mut u8> {
        unsafe { self.head_region.alloc(layout) }
    }

    /// Deallocate memory with the allocator
    #[inline]
    pub unsafe fn inner_deallocate(&self, ptr: *mut u8, layout: Layout) -> Result<()> {
        unsafe { self.head_region.dealloc(ptr, layout) }
    }
}

unsafe impl GlobalAlloc for QuantumHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { self.inner_allocate(layout).unwrap() }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.inner_deallocate(ptr, layout).unwrap() }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use core::alloc::Layout;
    extern crate std;

    #[test]
    fn test_buddy_new() {
        let len = 10 * 1024;
        let layout = Layout::from_size_align(len, 1).unwrap();
        let mem_region = unsafe { std::alloc::alloc_zeroed(layout) };

        let mut ptrs = std::vec::Vec::new();
        let mut alloc = BuddyAllocator::new(NonNull::new(mem_region).unwrap(), len);

        for i in 0..3 {
            let ptr = unsafe { alloc.alloc(Layout::new::<u8>()) }.unwrap();
            unsafe { *ptr = i };
            assert_eq!(unsafe { *ptr }, i);
            ptrs.push(ptr);
        }

        for i in 0..3 {
            let ptr = ptrs[i as usize];
            assert_eq!(unsafe { *ptr }, i);
            unsafe { alloc.dealloc(ptr, Layout::new::<u8>()) }.unwrap();
        }

        unsafe { std::alloc::dealloc(mem_region, layout) };
    }

    #[test]
    fn alloc_random() {
        let len = 32 * 1024;
        let layout = Layout::from_size_align(len, 1).unwrap();
        let mem_region = unsafe { std::alloc::alloc_zeroed(layout) };

        let mut ptrs = std::vec::Vec::new();
        let mut alloc = BuddyAllocator::new(NonNull::new(mem_region).unwrap(), len);

        for i in 0..100 {
            let ptr =
                unsafe { alloc.alloc(Layout::from_size_align((i * 8) % 128 + 8, 8).unwrap()) }
                    .unwrap();
            unsafe { *ptr = i as u8 };
            assert_eq!(unsafe { *ptr }, i as u8);
            ptrs.push(ptr);
        }

        for i in 0..100 {
            let ptr = ptrs[i as usize];
            assert_eq!(unsafe { *ptr }, i as u8);
            unsafe { alloc.dealloc(ptr, Layout::from_size_align((i * 8) % 128 + 8, 8).unwrap()) }
                .unwrap();
        }

        unsafe { std::alloc::dealloc(mem_region, layout) };
    }
}
