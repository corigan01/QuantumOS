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


use core::fmt::{Display, Formatter};
use core::mem::{align_of, size_of};
use core::ptr;
use core::ptr::NonNull;
use core::slice::Iter;
use over_stacked::raw_vec::RawVec;
use owo_colors::OwoColorize;
use quantum_utils::bytes::Bytes;
use crate::{AllocErr, ImproperConfigReason};
use crate::memory_layout::MemoryLayout;
use crate::usable_region::UsableRegion;

#[derive(Debug, Clone, Copy)]
pub enum DebugAllocationEvent {
    Free,
    Allocate,
    MakeNewVector
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use = "Not referencing ptr, will leak its allocation"]
pub struct UnsafeAllocationObject {
    pub ptr: u64,
    pub size: usize
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HeapEntryType {
    Free,
    Used,
    UsedByHeap,
    Forever
}

impl Display for HeapEntryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {

        let heap_entry_type_string = match self {
            HeapEntryType::Free => "Free",
            HeapEntryType::Used => "Used",
            HeapEntryType::UsedByHeap => "UsedByHeap",
            _ => "Unknown",
        };

        if !f.alternate() {
            write!(f, "{}", heap_entry_type_string)?;
        } else {
            match self {
                HeapEntryType::Free => {
                    write!(f, "{}", heap_entry_type_string.green().bold())?;
                },
                HeapEntryType::Used => {
                    write!(f, "{}", heap_entry_type_string.red())?;
                },
                HeapEntryType::UsedByHeap => {
                    write!(f, "{}", heap_entry_type_string.yellow())?;
                },
                _ => {
                    write!(f, "{}", heap_entry_type_string.red().bold())?;
                }
            }
        }

        let Some(width) = f.width() else {
            return Ok(());
        };

        let drawn_chars = heap_entry_type_string.chars().count();

        if drawn_chars > width {
            return Ok(());
        }

        let padding_to_draw = width - drawn_chars;

        for _ in 0..padding_to_draw {
            write!(f, " ")?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeapEntry {
    ptr: u64,
    pad: u32,
    size: u64,
    over: u64,
    kind: HeapEntryType
}

#[derive(Debug)]
pub struct KernelHeap {
    pub(crate) allocations: RawVec<HeapEntry>,
    total_allocated_bytes: usize,
    init_ptr: u64
}

impl KernelHeap {
    const VEC_ALLOC_MIN_SIZE_INCREASE: usize = 20;

    pub fn new(region: UsableRegion) -> Option<Self> {
        let init_vec_size = size_of::<HeapEntry>() * Self::VEC_ALLOC_MIN_SIZE_INCREASE;
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
            over: 0,
            kind: HeapEntryType::UsedByHeap
        };

        let init_alloc = HeapEntry {
            ptr: adjusted_ptr,
            pad: 0,
            size: adjusted_size,
            over: 0,
            kind: HeapEntryType::Free
        };

        raw_vec.push_within_capacity(used_section).unwrap();
        raw_vec.push_within_capacity(init_alloc).unwrap();

        Some(Self {
            allocations: raw_vec,
            total_allocated_bytes: region.size(),
            init_ptr: region.ptr().as_ptr() as u64,
        })
    }

    pub unsafe fn clear_entries(&mut self) {
        let size = self.total_allocated_bytes;
        let ptr = self.init_ptr;
        let region = UsableRegion::from_raw_parts(ptr as *mut u8, size).unwrap();

        *self = Self::new(region).unwrap();
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
        self.allocations.iter().for_each(|entry| {
            assert_ne!(entry.ptr, 0,
                       "in {:?}:: Buffer Health out-of-sync! We should not contain entries with ptrs of 0 in the allocator. ", caller);
            assert_ne!(entry.size, 0, "in {:#?}:: Buffer Health out-of-sync! We should not have entries with size of 0 in the allocator", caller);
            assert!(entry.ptr >= self.init_ptr, "in {:#?}:: Buffer Health out-of-sync! Below Minimum ptr found!", caller);
        });

        assert_ne!(self.allocations.iter().filter(|entry| {
            entry.kind == HeapEntryType::UsedByHeap
        }).count(), 0,
            "in {:?}:: Buffer Health no-allocations-ptr found in the allocator!", caller);

        let all_bytes: usize = self.allocations.iter().map(|entry| {
            entry.size as usize + entry.pad as usize
        }).sum();

        assert_eq!(all_bytes, self.total_allocated_bytes,
                   "in {:?}:: Buffer Health out-of-sync! Bytes in the buffer should equal bytes given!\nAllocations Dump: {:#?}", caller, self.allocations);
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

        Ok(())
    }

    pub unsafe fn allocate_impl(&mut self, allocation_description: MemoryLayout, avoid_vec_safety_check: bool, mark_allocated_as: HeapEntryType)
                                -> Result<UnsafeAllocationObject, AllocErr> {
        if self.allocations.remaining() <= 3 && !avoid_vec_safety_check {
            self.reallocate_internal_vector()?;
        }

        let requested_bytes = allocation_description.bytes();
        let requested_align = allocation_description.alignment();

        if requested_bytes == 0 {
            return Err(AllocErr::ImproperConfig(ImproperConfigReason::ZeroSize));
        }
        if requested_align == 0 || (requested_align != 1 && !requested_align.is_power_of_two()) {
            return Err(AllocErr::ImproperConfig(ImproperConfigReason::AlignmentInvalid(requested_align)));
        }

        let best_entry_info = self.allocations
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                let (_, bytes_to_align) = Self::align_ptr(entry.ptr, requested_align);
                entry.size as usize >= (requested_bytes + bytes_to_align) && entry.kind == HeapEntryType::Free
            })
            .min_by_key(|(_, entry)| entry.size as usize)
            .map(|(id, _)| (id, self.allocations[id]));

        let Some((best_entry_id, best_entry)) = best_entry_info else {
            return Err(AllocErr::OutOfMemory(requested_bytes, self.total_bytes_of(HeapEntryType::Free)));
        };

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
                over: 0,
                kind: HeapEntryType::Free
            };

            let modified_old_entry = HeapEntry {
                ptr: best_entry_new_ptr,
                pad: 0,
                size: best_entry.size - (best_entry_align_bump as u64),
                over: (best_entry.size - (best_entry_align_bump as u64)) - requested_bytes as u64,
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
                over: 0,
                kind: HeapEntryType::Free
            };

            let ptr = if pre_allocation_split { best_entry_new_ptr }
                else { best_entry.ptr };

            let pad = if pre_allocation_split { 0 }
                else { best_entry_align_bump as u32 };

            let modified_old_entry = HeapEntry {
                ptr,
                pad,
                size: requested_bytes as u64,
                over: 0,
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
            let best_entry_mut = self.allocations.get_mut(best_entry_id).unwrap();
            if best_entry_mut.size < requested_bytes as u64 ||
                best_entry_mut.kind != HeapEntryType::Free {
                return Err(AllocErr::InternalErr);
            }

                        best_entry_mut.size -= best_entry_align_bump as u64;
            best_entry_mut.pad = best_entry_align_bump as u32;
            best_entry_mut.kind = mark_allocated_as;
            best_entry_mut.over = best_entry_mut.size - requested_bytes as u64;
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

    pub unsafe fn impl_realloc<Type>(&mut self, old_ptr: NonNull<Type>, new_alloc_desc: MemoryLayout, can_skip_fill: bool, fill: u8) -> Result<UnsafeAllocationObject, AllocErr> {
        let searching_ptr = old_ptr.as_ptr() as u64;

        let Some(old_alloc) =
            self.allocations.iter().find(|entry| {
                (entry.ptr + (entry.pad as u64)) == searching_ptr
            }) else {
            return Err(AllocErr::NotFound(searching_ptr as usize));
        };
        let old_alloc = *old_alloc;
        if (old_alloc.size - old_alloc.over) > new_alloc_desc.bytes() as u64 {
            return Err(AllocErr::ImproperConfig(
                ImproperConfigReason::Smaller(old_alloc.size as usize, new_alloc_desc.bytes()))
            );
        }


        let new_alloc = self.allocate(new_alloc_desc)?;
        let new_alloc_ptr = new_alloc.ptr as *mut u8;

        if !can_skip_fill {
            ptr::write_bytes(new_alloc_ptr.add(old_alloc.size as usize), fill, new_alloc.size - (old_alloc.size - old_alloc.over) as usize);
        }

        ptr::copy(searching_ptr as *const u8, new_alloc_ptr, old_alloc.size as usize);

        self.free(old_ptr)?;

        Ok(new_alloc)
    }

    pub unsafe fn realloc<Type>(&mut self, old_ptr: NonNull<Type>, new_alloc_desc: MemoryLayout) -> Result<UnsafeAllocationObject, AllocErr> {
        self.impl_realloc(old_ptr, new_alloc_desc, true, 0)
    }

    pub unsafe fn realloc_fill<Type>(&mut self, old_ptr: NonNull<Type>, new_alloc_desc: MemoryLayout, fill: u8) -> Result<UnsafeAllocationObject, AllocErr> {
        self.impl_realloc(old_ptr, new_alloc_desc, false, fill)
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
                self.allocations[index].over = 0;
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
                (entry.ptr + (entry.pad as u64)) == searching_ptr
            });

        let mut did_find = false;
        for entry in allocations_with_ptr {
            if entry.kind == HeapEntryType::Free {
                return Err(AllocErr::DoubleFree(ptr.as_ptr() as usize));
            }

            entry.kind = HeapEntryType::Free;
            entry.size += entry.pad as u64;
            entry.pad = 0;
            entry.over = 0;

            did_find = true;
            break;
        }

        if !did_find {
            return Err(AllocErr::NotFound(ptr.as_ptr() as usize));
        }

        self.consolidate_entries();
        self.ensure_buffer_health(DebugAllocationEvent::Free);

        Ok(())
    }
}

impl Display for KernelHeap {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let bytes_free = self.total_bytes_of(HeapEntryType::Free);
        let percent_free = (bytes_free as f64) / (self.total_allocated_bytes as f64);

        writeln!(f, "Heap: {} {} ({:.2}%)",
            Bytes::from(bytes_free).green().bold(),
            "Free".green().bold(),
                 (percent_free * 100.0),
        )?;

        writeln!(f, "|       Ptr      |    Size    |    Loss    |    Type    |  Alloc % |")?;
        writeln!(f, "+----------------+------------+------------+------------+----------+")?;
        for region in self.allocations.iter() {
            let start = region.ptr + region.pad as u64;
            let size = region.size;
            let loss = region.pad as u64 + region.over;
            let kind = region.kind;
            let percent = ((size + loss) as f64) / (self.total_allocated_bytes as f64);

            writeln!(f, "| 0x{:012x} | {:10} | {:10} | {:#10} | {:7.4}% |",
                start,
                Bytes::from(size),
                Bytes::from(loss),
                kind,
                percent * 100.0
            )?;
        }
        writeln!(f, "+----------------+------------+------------+------------+----------+")?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    extern crate alloc;

    use alloc::alloc::{alloc, dealloc};
    use core::alloc::Layout;
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

    const ALLOCATION_BUFFER_SIZE: usize = 4096;

    fn get_example_allocator() -> KernelHeap {
        let memory_layout = Layout::from_size_align(ALLOCATION_BUFFER_SIZE, 1).unwrap();

        let nonnull = unsafe {
            let alloc = alloc(memory_layout);

             NonNull::new(alloc).unwrap()
        };

        let usable_region =
            unsafe { UsableRegion::from_raw_parts(nonnull.as_ptr(), ALLOCATION_BUFFER_SIZE) }
                .unwrap();

        KernelHeap::new(usable_region).unwrap()
    }

    fn free_my_example_allocator(heap: KernelHeap) {
        let memory_layout = Layout::from_size_align(ALLOCATION_BUFFER_SIZE, 1).unwrap();

        unsafe {
            dealloc(heap.init_ptr as *mut u8, memory_layout);
        }
    }

    #[test]
    fn construct_a_new_allocator() {
        let allocator = get_example_allocator();

        assert_ne!(allocator.allocations.len(), 0);

        free_my_example_allocator(allocator);
    }

    #[test]
    fn test_one_allocation() {
        let mut allocator = get_example_allocator();
        let example_layout = MemoryLayout::from_type::<u8>();
        let allocation_result = unsafe { allocator.allocate(example_layout) };

        assert!(allocation_result.is_ok());

        free_my_example_allocator(allocator);
    }

    #[test]
    fn test_5_one_byte_allocations() {
        let mut allocator = get_example_allocator();
        let example_layout = MemoryLayout::from_type::<u8>();

        for _ in 0..5 {
            let allocation_result = unsafe { allocator.allocate(example_layout) };
            assert!(allocation_result.is_ok(), "Allocator was not able to allocate 5 regions of 1 byte, result was {:#?}", allocation_result);
        }

        free_my_example_allocator(allocator);
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
        free_my_example_allocator(allocator);
    }

    #[test]
    fn test_free_with_allocation() {
        let mut allocator = get_example_allocator();

        type TestAllocType = u64;

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

        free_my_example_allocator(allocator);
    }

    extern crate test;
    use test::Bencher;

    #[bench]
    fn test_lots_of_allocation(b: &mut Bencher) {
        b.iter(|| {
            let mut allocation_data: [u8; 1024] = [0; 1024];
            let usable_region = UsableRegion::new(&mut allocation_data);
            let mut allocator = KernelHeap::new(usable_region).unwrap();

            type TestAllocType = u64;

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

        free_my_example_allocator(allocator);
    }

    #[test]
    fn test_realloc() {
        let mut allocator = get_example_allocator();

        type TestAllocType = u64;
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

        let new_layout = MemoryLayout::new(example_layout.alignment(), example_layout.bytes() * 2);

        let new_alloc = unsafe { allocator.realloc(nonnull_ptr, new_layout) };
        assert!(new_alloc.is_ok());
        let new_alloc = new_alloc.unwrap();

        let nonnull_ptr = NonNull::new(new_alloc.ptr as *mut u64).unwrap();
        assert_eq!(unsafe {*nonnull_ptr.as_ptr()}, 0xBABBEEF);

        let free_result = unsafe { allocator.free(nonnull_ptr) };
        assert_eq!(free_result, Ok(()));

        free_my_example_allocator(allocator);
    }

}