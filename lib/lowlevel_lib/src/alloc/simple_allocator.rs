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

/// A simple bump allocator that uses a region of memory to allocate
/// variable-sized memory regions.
///
/// The `SimpleBumpAllocator` works by taking a mutable slice of memory
/// and allocating memory regions by bumping a pointer to the next free
/// byte of memory within the slice. It's a very basic allocator and
/// lacks many of the more advanced features of modern allocators,
/// but it can be useful in situations where you need a simple and
/// efficient allocator that's easy to reason about.
///
/// # Safety
///
/// This allocator is not thread-safe and does not provide any form
/// of memory safety or protection. It's up to the user to ensure that
/// they use this allocator safely and correctly, and to avoid common
/// memory allocation bugs like buffer overflows and use-after-free errors.
pub struct SimpleBumpAllocator<'a> {
    memory_slice: &'a mut [u8],
    used_memory: usize,
}

impl<'a> SimpleBumpAllocator<'a> {
    /// Creates a new `SimpleBumpAllocator` that will allocate memory regions from the given memory slice.
    ///
    /// # Arguments
    /// * `memory_region` - A mutable reference to the slice of memory that this allocator will allocate from.
    ///
    /// # Examples
    /// ```
    /// use quantum_lib::alloc::simple_allocator::SimpleBumpAllocator;
    ///
    /// let mut memory = [0u8; 1024];
    /// let allocator = SimpleBumpAllocator::new(&mut memory);
    /// ```
    pub fn new(memory_region: &'a mut [u8]) -> Self {
        Self {
            memory_slice: memory_region,
            used_memory: 0,
        }
    }

    /// Creates a new `SimpleBumpAllocator` that will allocate memory regions from the memory at the given pointer.
    ///
    /// # Safety
    /// The caller must ensure that the pointer is valid and points to a block of memory that is at least `size` bytes long.
    ///
    /// # Arguments
    /// * `ptr` - A raw pointer to the start of the memory region that this allocator will allocate from.
    /// * `size` - The size of the memory region that this allocator will allocate from.
    ///
    /// # Returns
    /// Returns `Some(allocator)` if the allocator was successfully created and verified to be working.
    /// Otherwise, returns `None`.
    ///
    /// # Examples
    /// ```
    /// use core::mem::size_of;
    /// use quantum_lib::alloc::simple_allocator::SimpleBumpAllocator;
    ///
    /// let mut memory = [0u8; 1024];
    ///
    /// let ptr = memory.as_mut_ptr();
    /// let allocator = unsafe { SimpleBumpAllocator::new_from_ptr(ptr, memory.len()) };
    /// ```
    pub unsafe fn new_from_ptr(ptr: *mut u8, size: usize) -> Self {
        Self {
            memory_slice: core::slice::from_raw_parts_mut(ptr, size),
            used_memory: 0,
        }
    }

    /// Allocates a new region of memory of the specified size.
    ///
    /// If there is enough unused memory in the `memory_slice` field of the `SimpleBumpAllocator`
    /// instance, a new slice of `u8` values of the requested size is allocated from the unused
    /// memory, and a mutable reference to the slice is returned.
    ///
    /// If the requested size is larger than the amount of unused memory in the `memory_slice`,
    /// or if there is no unused memory remaining, then `None` is returned.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the memory region to allocate, in bytes.
    ///
    /// # Safety
    ///
    /// This method uses unsafe code to create a mutable reference to a slice of memory. It is the
    /// responsibility of the caller to ensure that the returned reference is not used after the
    /// memory it points to has been deallocated or reused. Additionally, the `SimpleBumpAllocator`
    /// struct is not thread-safe and cannot be used concurrently from multiple threads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use quantum_lib::alloc::simple_allocator::SimpleBumpAllocator;
    ///
    /// let mut memory = [0u8; 1024];
    /// let mut allocator = SimpleBumpAllocator::new(&mut memory);
    ///
    /// let region = allocator.allocate_region(512);
    /// assert!(region.is_some());
    /// assert_eq!(region.unwrap().len(), 512);
    ///
    /// let region2 = allocator.allocate_region(1024);
    /// assert!(region2.is_none());
    /// ```
    pub fn allocate_region(&mut self, size: usize) -> Option<&'a mut [u8]> {
        if self.used_memory + size > self.memory_slice.len() {
            return None;
        }

        let slice = unsafe {
            core::slice::from_raw_parts_mut(
                self.memory_slice.as_mut_ptr().add(self.used_memory),
                size,
            )
        };

        self.used_memory += size;

        Some(slice)
    }
}

#[cfg(test)]
mod tests {
    use crate::alloc::simple_allocator::SimpleBumpAllocator;

    #[test]
    fn test_allocate_region() {
        let mut binding = [0; 1024];
        let mut allocator = SimpleBumpAllocator::new(&mut binding);

        assert!(allocator.allocate_region(512).is_some());
        assert!(allocator.allocate_region(512).is_some());
        assert!(allocator.allocate_region(1).is_none());
    }

    #[test]
    fn test_new_from_ptr() {
        let mut memory = [0; 1024];

        let ptr = memory.as_mut_ptr();
        let size = memory.len();

        let mut allocator = unsafe { SimpleBumpAllocator::new_from_ptr(ptr, size) };

        assert!(allocator.allocate_region(512).is_some());
        assert!(allocator.allocate_region(512).is_some());
        assert!(allocator.allocate_region(1).is_none());
    }
}
