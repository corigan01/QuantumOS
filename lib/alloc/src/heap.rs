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


use core::iter::{Filter, FilterMap};
use core::mem::{align_of, size_of};
use core::ptr::NonNull;
use core::slice::{Iter, IterMut};
use over_stacked::raw_vec::RawVec;
use crate::AllocErr;
use crate::memory_layout::MemoryLayout;
use crate::usable_region::UsableRegion;

#[derive(Debug, Clone, Copy)]
pub enum DebugAllocationEvent {
    Free,
    Allocate,
    MakeNewVector
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnsafeAllocationObject {
    ptr: u64,
    size: usize
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HeapEntryType {
    Free,
    Used,
    UsedByHeap,
    Forever
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeapEntry {
    ptr: u64,
    pad: u32,
    size: u64,
    kind: HeapEntryType
}

pub struct KernelHeap {
    allocations: RawVec<HeapEntry>,
    total_allocated_bytes: usize
}

impl KernelHeap {
    const VEC_ALLOC_MIN_SIZE_INCREASE: usize = 10;

    pub fn new(region: UsableRegion) -> Option<Self> {
        let init_vec_size = (size_of::<HeapEntry>() * Self::VEC_ALLOC_MIN_SIZE_INCREASE) + 1;
        if region.size() <= init_vec_size {
            return None;
        }

        let region_start_ptr = region.ptr();
        let (aligned_ptr, bytes_to_align) =
            Self::align_ptr(region_start_ptr.as_ptr() as u64, align_of::<HeapEntry>());

        let aligned_nonnull = NonNull::new(aligned_ptr as *mut HeapEntry).unwrap();

        let mut raw_vec: RawVec<HeapEntry> = RawVec::begin(aligned_nonnull, 10);

        let adjusted_ptr = region_start_ptr.as_ptr() as u64 + init_vec_size as u64 + bytes_to_align as u64;
        let adjusted_size = (region.size() - (init_vec_size + bytes_to_align)) as u64;

        let used_section = HeapEntry {
            ptr: region.ptr().as_ptr() as u64,
            pad: bytes_to_align as u32,
            size: init_vec_size as u64,
            kind: HeapEntryType::UsedByHeap
        };

        let init_alloc = HeapEntry {
            ptr: adjusted_ptr,
            pad: 0,
            size: adjusted_size,
            kind: HeapEntryType::Free
        };

        raw_vec.push_within_capacity(used_section).unwrap();
        raw_vec.push_within_capacity(init_alloc).unwrap();

        Some(Self {
            allocations: raw_vec,
            total_allocated_bytes: region.size(),
        })
    }

    fn align_ptr(ptr: u64, alignment: usize) -> (u64, usize) {
        if alignment == 1 {
            return (ptr, 0);
        }

        let ptr_casted = ptr as *const u8;
        let alignment_offset = ptr_casted.align_offset(alignment);
        (ptr + (alignment_offset as u64), alignment_offset)
    }

    fn ensure_buffer_health(&self, caller: DebugAllocationEvent) {
        let all_bytes: usize = self.allocations.iter().map(|entry| {
            entry.size as usize + entry.pad as usize
        }).sum();

        assert_eq!(all_bytes, self.total_allocated_bytes,
                   "{:?}:: Buffer Health out-of-sync! Bytes in the buffer should equal bytes given!\nAllocations Dump: {:#?}", caller, self.allocations);
    }

    pub fn total_bytes_of(&self, kind: HeapEntryType) -> usize {
        self.allocations.iter().filter_map(|entry| {
            if entry.kind != kind {
                None
            } else {
                Some(entry.size as usize)
            }
        }).sum()
    }

    pub fn max_continuous_of(&self, kind: HeapEntryType) -> usize {
        self.allocations.iter().filter_map(|entry| {
            if entry.kind != kind {
                None
            } else {
                Some(entry.size as usize)
            }
        }).max().unwrap_or(0)
    }

    pub fn allocations_iter(&self) -> Iter<HeapEntry> {
        self.allocations.iter()
    }

    pub fn reallocate_internal_vector(&mut self) -> Result<(), AllocErr> {
        let new_entry_qty =
            ((self.allocations.capacity() / Self::VEC_ALLOC_MIN_SIZE_INCREASE) + 1) * Self::VEC_ALLOC_MIN_SIZE_INCREASE;

        let memory_layout_for_internal_vector =
            MemoryLayout::new(align_of::<HeapEntry>(), size_of::<HeapEntry>() * new_entry_qty);
        let allocation =
            unsafe { self.allocate_impl(memory_layout_for_internal_vector, true, HeapEntryType::UsedByHeap) }?;

        let old_ptr: NonNull<HeapEntry> =
            self.allocations.grow(NonNull::new(allocation.ptr as *mut HeapEntry).unwrap(), new_entry_qty)
                .map_err(|_e| { AllocErr::InternalErr } )?;

        unsafe { self.free(old_ptr).expect("Free Should not fail for old Vector!") };

        self.ensure_buffer_health(DebugAllocationEvent::MakeNewVector);

        Ok(())
    }

    pub unsafe fn allocate_impl(&mut self, allocation_description: MemoryLayout, avoid_vec_safety_check: bool, mark_allocated_as: HeapEntryType)
                                -> Result<UnsafeAllocationObject, AllocErr> {
        let requested_bytes = allocation_description.bytes();
        let requested_align = allocation_description.alignment();

        if requested_bytes == 0 {
            return Err(AllocErr::ImproperConfig);
        }
        if requested_align == 0 || (requested_align != 1 && !requested_align.is_power_of_two()) {
            return Err(AllocErr::ImproperConfig);
        }

        let proper_sized_free_allocation_iter =
            self.allocations.iter().enumerate().filter(|(_, entry)| {
                entry.kind == HeapEntryType::Free && (entry.size as usize) >= requested_bytes
            });

        let proper_size_with_alignment_iter = proper_sized_free_allocation_iter.filter(|(_, entry) | {
            let (_, bytes_to_align) = Self::align_ptr(entry.ptr, requested_align);

            (entry.size as usize - bytes_to_align) >= requested_bytes
        });

        let mut best_entry: Option<(usize, HeapEntry)> = None;

        for (entry_id, entry_ref) in proper_size_with_alignment_iter {
            assert_eq!(entry_ref.kind, HeapEntryType::Free,
                       "It should be impossible for a non-free entry to be iterated in this for loop!");

            let working_ptr = entry_ref.ptr;

            let (_, bytes_to_alignment) =
                Self::align_ptr(working_ptr, requested_align);

            if best_entry.is_none() {
                best_entry = Some((entry_id, entry_ref.clone()));
                continue;
            }

            let Some((_, some_best_entry)) = best_entry else {
                unreachable!("We just ensured that `smallest_allocation_possible` is_some()!");
            };

            let (_, their_bytes_to_alignment) =
                Self::align_ptr(some_best_entry.ptr, requested_align);

            let our_bytes = entry_ref.size as usize;
            let their_bytes = some_best_entry.size as usize;

            let our_score = bytes_to_alignment + our_bytes;
            let their_score = their_bytes_to_alignment + their_bytes;

            // We want the lowest score
            if our_score > their_score {
                best_entry = Some((entry_id, entry_ref.clone()));
            }
        }

        let Some((best_entry_id, best_entry)) = best_entry else {
            return Err(AllocErr::OutOfMemory);
        };

        // We need to ensure that our RawVec here has enough space to allocate,
        // our possibly 2 new regions. So we need to assert that our RawVec has space
        // for 5 allocations, if it does not, then we need to reallocate RawVec.
        if self.allocations.remaining() <= 3 && !avoid_vec_safety_check {
            self.reallocate_internal_vector()?;
        }

        // We might have to split our entry into 3 if we have a perfect situation.
        // Say we have an allocation that is terribly aligned, and we need a *very*
        // aligned allocation (4k for example). We could lose up to 'alignment - 1'
        // bytes! This could be huge, so we could try to split the allocation so that
        // we can make a new free allocation out of the missed alignment.

        let (best_entry_new_ptr, best_entry_align_bump) =
            Self::align_ptr(best_entry.ptr, requested_align);

        let pre_allocation_split = best_entry_align_bump > (size_of::<HeapEntry>() * 2);
        let post_allocation_split = best_entry.size as usize > (requested_bytes + size_of::<HeapEntry>() * 2);

        // We need to split to save the alignment
        if pre_allocation_split {
            let before_split_entry = HeapEntry {
                ptr: best_entry.ptr,
                pad: 0,
                size: (best_entry_align_bump - 1) as u64,
                kind: HeapEntryType::Free
            };

            let modified_old_entry = HeapEntry {
                ptr: best_entry_new_ptr,
                pad: 0,
                size: best_entry.size - (best_entry_align_bump as u64),
                kind: mark_allocated_as
            };

            if self.allocations.replace(best_entry_id, modified_old_entry).is_err() {
                return Err(AllocErr::InternalErr);
            }
            if self.allocations.insert(best_entry_id + 1, before_split_entry).is_err() {
                return Err(AllocErr::InternalErr);
            }

        }

        // We need to split to save the post-allocated-area
        if post_allocation_split {
            let after_split_entry = HeapEntry {
                ptr: best_entry_new_ptr + (requested_bytes as u64),
                pad: 0,
                size: best_entry.size - ((best_entry_align_bump + requested_bytes) as u64),
                kind: HeapEntryType::Free
            };

            let ptr = if pre_allocation_split { best_entry_new_ptr }
            else { best_entry.ptr };

            let modified_old_entry = HeapEntry {
                ptr,
                pad: best_entry_align_bump as u32,
                size: requested_bytes as u64,
                kind: mark_allocated_as
            };


            if self.allocations.replace(best_entry_id, modified_old_entry).is_err() {
                return Err(AllocErr::InternalErr);
            }
            if self.allocations.insert(best_entry_id + 1, after_split_entry).is_err() {
                return Err(AllocErr::InternalErr);
            }
        }

        // Change the main entry to be reserved
        if !(post_allocation_split || pre_allocation_split) {
            self.allocations[best_entry_id].kind = mark_allocated_as;
        }

        // Construct the output
        let returning_value = self.allocations[best_entry_id];

        assert_ne!(best_entry_new_ptr, 0, "Somehow we allocated a null ptr!");
        assert_ne!(returning_value.size, 0, "Somehow we allocated a null size!");

        self.ensure_buffer_health(
            if avoid_vec_safety_check { DebugAllocationEvent::MakeNewVector }
            else { DebugAllocationEvent::Allocate}
        );

        Ok(UnsafeAllocationObject {
            ptr: best_entry_new_ptr,
            size: requested_bytes
        })
    }

    pub unsafe fn allocate(&mut self, allocation_description: MemoryLayout) -> Result<UnsafeAllocationObject, AllocErr> {
        self.allocate_impl(allocation_description, false, HeapEntryType::Used)
    }

    pub fn consolidate_entries(&mut self) {
        let mut index = 0;
        while index <= self.allocations.len() {
            let Some(entry) = self.allocations.get_ref(index) else {
                break;
            };
            let Some(next_entry) = self.allocations.get_ref(index + 1) else {
                break;
            };

            if entry.kind != HeapEntryType::Free || next_entry.kind != HeapEntryType::Free {
                index += 1;
                continue;
            }

            let entry_end = entry.ptr + (entry.pad as u64) + entry.size;

            if entry_end == next_entry.ptr {
                self.allocations[index].size = entry.size + next_entry.size;
                self.allocations[index].pad = 0;
                self.allocations.remove(index + 1);
                continue;
            }

            index += 1;

        }
    }

    pub unsafe fn free<Type>(&mut self, ptr: NonNull<Type>) -> Result<(), AllocErr> {
        let searching_ptr = ptr.as_ptr() as u64;

        let allocations_with_ptr =
            self.allocations.mut_iter().filter(|entry| {
                entry.ptr != searching_ptr && (entry.ptr + (entry.pad as u64)) != searching_ptr
            });

        let mut did_find = false;
        for entry in allocations_with_ptr {
            if entry.kind == HeapEntryType::Free {
                assert!(false, "This: {:#?}\nall: {:#?}", entry.clone(), self.allocations);
                return Err(AllocErr::DoubleFree);
            }

            entry.kind = HeapEntryType::Free;
            did_find = true;
            break;
        }

        if !did_find {
            return Err(AllocErr::NotFound);
        }

        self.consolidate_entries();
        self.ensure_buffer_health(DebugAllocationEvent::Free);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use core::mem::align_of;
    use super::*;

    #[test]
    fn test_align_ptr() {
        let alignment = 8;
        let ptr = 100;

        let (aligned_ptr, offset) = KernelHeap::align_ptr(ptr, alignment);
        assert_eq!(aligned_ptr % alignment as u64, 0);
        assert_eq!(aligned_ptr, ptr + offset as u64);

        let alignment = 16;
        let ptr = 200;

        let (aligned_ptr, offset) = KernelHeap::align_ptr(ptr, alignment);
        assert_eq!(aligned_ptr % alignment as u64, 0);
        assert_eq!(aligned_ptr, ptr + offset as u64);
    }

    #[test]
    #[should_panic]
    fn test_align_ptr_zero() {
        let alignment = 0;
        let ptr = 123;

        KernelHeap::align_ptr(ptr, alignment);
    }

    #[test]
    fn test_align_ptr_already_aligned() {
        let alignment = 16;
        let ptr = 128;

        let (aligned_ptr, offset) = KernelHeap::align_ptr(ptr, alignment);
        assert_eq!(aligned_ptr, ptr);
        assert_eq!(offset, 0);
    }

    static mut ALLOCATION_SPACE: [u8; 4096] = [0; 4096];

    fn get_example_allocator() -> KernelHeap {
        unsafe { ALLOCATION_SPACE = [0; 4096]; }
        let usable_region = UsableRegion::new(unsafe { &mut ALLOCATION_SPACE });
        KernelHeap::new(usable_region).unwrap()
    }

    #[test]
    fn construct_a_new_allocator() {
        let allocator = get_example_allocator();

        assert_ne!(allocator.allocations.len(), 0);
    }

    #[test]
    fn test_one_allocation() {
        let mut allocator = get_example_allocator();
        let example_layout = MemoryLayout::from_type::<u8>();
        let allocation_result = unsafe { allocator.allocate(example_layout) };

        assert!(allocation_result.is_ok());

    }

    #[test]
    fn test_5_one_byte_allocations() {
        let mut allocator = get_example_allocator();
        let example_layout = MemoryLayout::from_type::<u8>();

        for _ in 0..5 {
            let allocation_result = unsafe { allocator.allocate(example_layout) };
            assert!(allocation_result.is_ok(), "Allocator was not able to allocate 5 regions of 1 byte, result was {:#?}", allocation_result);
        }
    }

    #[test]
    fn test_one_aligned_allocation() {
        let mut allocator = get_example_allocator();

        type TestAllocType = u64;

        let test_allocation_type_alignment = align_of::<TestAllocType>();
        let example_layout = MemoryLayout::from_type::<TestAllocType>();

        let allocation = unsafe { allocator.allocate(example_layout) };
        assert!(allocation.is_ok());

        let Ok(allocation) = allocation else {
            unreachable!();
        };

        assert_eq!(allocation.size, size_of::<TestAllocType>());
        assert_eq!((allocation.ptr as *const ()).align_offset(test_allocation_type_alignment), 0);
    }

    #[test]
    fn test_free_with_allocation() {
        let mut allocator = get_example_allocator();

        type TestAllocType = u64;

        let test_allocation_type_alignment = align_of::<TestAllocType>();
        let example_layout = MemoryLayout::from_type::<TestAllocType>();

        let allocation = unsafe { allocator.allocate(example_layout) };
        assert!(allocation.is_ok());

        let Ok(allocation) = allocation else {
            unreachable!();
        };


        let nonnull_ptr = NonNull::new(allocation.ptr as *mut u64).unwrap();

        unsafe {
            *nonnull_ptr.as_ptr() = 0xBABBEEF;
        };

        assert_eq!(unsafe {*nonnull_ptr.as_ptr()}, 0xBABBEEF);

        let free_result = unsafe { allocator.free(nonnull_ptr) };

        assert_eq!(free_result, Ok(()));
    }

    extern crate test;
    use test::{Bencher, black_box};

    #[bench]
    fn test_lots_of_allocation(b: &mut Bencher) {
        b.iter(|| {
            let mut allocation_data: [u8; 1024] = [0; 1024];
            let usable_region = UsableRegion::new(&mut allocation_data);
            let mut allocator = KernelHeap::new(usable_region).unwrap();

            type TestAllocType = u64;

            let test_allocation_type_alignment = align_of::<TestAllocType>();
            let example_layout = MemoryLayout::from_type::<TestAllocType>();

            let allocation = unsafe { allocator.allocate(example_layout) };
            assert!(allocation.is_ok());
            assert_ne!(allocator.total_bytes_of(HeapEntryType::Free), 0, "We are allocating, and freeing, there should not be memory loss!");

            let Ok(allocation) = allocation else {
                unreachable!();
            };

            let nonnull_ptr = NonNull::new(allocation.ptr as *mut u64).unwrap();

            unsafe {
                *nonnull_ptr.as_ptr() = 0xBABBEEF;
            };

            assert_eq!(unsafe {*nonnull_ptr.as_ptr()}, 0xBABBEEF);

            let free_result = unsafe { allocator.free(nonnull_ptr) };

            assert_eq!(free_result, Ok(()), "{:#?}", allocator.allocations);
        })
    }

    #[test]
    fn make_over_10_allocations() {
        let mut allocator = get_example_allocator();
        let example_layout = MemoryLayout::from_type::<u8>();

        for _ in 0..20 {
            let allocation_result = unsafe { allocator.allocate(example_layout) };
            assert!(allocation_result.is_ok(), "Allocator was not able to allocate 20 regions of 1 byte, result was {:#?}", allocation_result);
        }

        assert!(allocator.total_bytes_of(HeapEntryType::Free) >= 4096,
                "{:#?}", allocator.allocations);
    }

}