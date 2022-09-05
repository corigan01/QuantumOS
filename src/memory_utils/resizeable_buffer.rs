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

use heapless::Vec;
use crate::memory_utils::safe_ptr::SafePtr;
use crate::memory_utils::safe_size::SafeSize;

struct BufferComponent<T> {
    ptr: SafePtr<T>,
    capacity: SafeSize,
    used: usize,
}

pub struct ResizeableBuffer<T> {
    /// # Internal Buffer
    /// We can store up to 255 pointers with differing sizes before we overflow, but that should
    /// be more then enough because each pointer can store tons of memory at a time.
    /// ## Scaling
    /// Each time the buffer expands it expands experimentally with the amount of elements. The first
    /// allocation might only have 1k in storage, but as we allocate more and more memory to this
    /// buffer we know that we are going to need more memory to hold it all.
    /// ## Example of allocation
    ///
    /// ```text
    /// ELEMENTS: [========================            ][...] 24/40 elements are filled
    ///
    /// | First Allocation | |       Second Allocation      | |        Final Allocation        |
    /// [==================] [==============================] [=======                         ]
    ///       100 bytes                  150 bytes                          200 bytes
    /// [=============================================================                         ]
    ///                                      275/450 bytes used
    /// Total memory used: 450 bytes
    /// Memory with data: 275 bytes
    /// Efficiency: 60%
    ///
    /// ```
    ///
    /// ## Freeing allocations
    /// If the buffer is given the ability to free its allocations; then we will look for when we hit
    /// 60% usage on the previous allocation to free the latest allocation.
    ///
    /// This gives us the most efficiency with not allocating / freeing too much memory at for small
    /// changes to the vector.
    internal_buffer: Option<Vec<BufferComponent<T>, 255>>,

    /// # Total Capacity
    /// This is the capacity in elements that can be pushed into the current size of the buffer.
    ///
    /// # Why this?
    /// This allows us to quickly determine if the number of elements in the buffer without
    /// iterating over each element to check if its freed or not. Each `BufferComponent` also
    /// contains a size vector for its type.
    ///
    /// # Safety
    /// This element uses `SafeSize` to ensure that the value isn't defined unless its above 0. We
    /// do this to make sure that the capacity is not defined for states that are not valid like
    /// having a capacity of zero!
    total_capacity: SafeSize,

    /// # Used Elements
    /// This value is defined to the amount of elements that currently are populated with a value.
    /// Each `BufferComponent` also has its own used_elements variable to speed up the lookup
    /// process of knowing how much we have allocated, and where all those allocations reside.
    ///
    /// # Future
    /// When the used elements in each `BufferComponent` drop below a defined value, we should drop
    /// the allocation, and move each element to a different `BufferComponent`.
    used_elements: usize,

    // total `BufferComponent`
    total_allocations: SafeSize,

    /// the percentage that we free `BufferComponent`'s
    to_free_percentage: SafeSize,

}

impl<T> ResizeableBuffer<T> {
    pub fn new() -> Self {
        Self {
            internal_buffer: None,
            total_capacity: SafeSize::new(),
            used_elements: 0,
            total_allocations: SafeSize::new(),
            to_free_percentage: SafeSize::new()
        }
    }
}

pub fn test() {
    let buffer: ResizeableBuffer<i32> = ResizeableBuffer::new();
}