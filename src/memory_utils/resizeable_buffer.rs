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

use core::mem::size_of;
use heapless::Vec;
use crate::{debug_println, serial_print, serial_println};
use crate::memory::VirtualAddress;
use crate::memory_utils::safe_ptr::SafePtr;
use crate::memory_utils::safe_size::SafeSize;
use crate::error_utils::QuantumError;

#[derive(Debug)]
struct BufferComponent<T> {
    ptr: SafePtr<(T, Option<usize>)>,
    capacity: SafeSize,
    used: usize,
    key_offset: usize,
}

impl<T> BufferComponent<T> {
    pub fn new() -> Self {
        Self {
            ptr: SafePtr::new(),
            capacity: SafeSize::new(),
            used: 0,
            key_offset: 0
        }
    }

    pub fn set_element(&mut self, element: T, key: usize) -> Result<(), QuantumError> {
        // Check if the key is valid within our range
        if key < self.key_offset {
            return Err(QuantumError::NoItem)
        }

        // check if we already have an element defined with the given key
        if let Some(value) = self.get_element_with_key(key) {
            *value = element;
        } else if let Some(pointer) = self.ptr.as_ptr() {
            // Check if the capacity allows us to set a new element
            if let Some(capacity) = self.capacity.get() {
                if self.used >= capacity {
                    return Err(QuantumError::BufferFull);
                }
            } else {
                return Err(QuantumError::UndefinedValue);
            }

            // Finally if we know we *dont* have an element with the same key, and we have space
            // for another allocation, then we can append the element and key to the buffer
            let real_pointer = unsafe {
                pointer.add(key)
            };

            unsafe { *real_pointer = (element, Some(key)) };
            self.used += 1;

        } else {
            return Err(QuantumError::NoItem);
        }

        Ok(())
    }

    pub fn remove_element(&mut self, key: usize) -> Result<T, QuantumError> {
        if let Some(existing_element) = self.get_element_with_key(key) {
            if let Some(self_ptr) = self.ptr.as_ptr() {

            }


        }

        Err(QuantumError::NoItem)
    }

    pub fn does_contain_element(&self, key: usize) -> bool {
        self.get_element_with_key(key).is_some()
    }

    pub fn get_element_with_key(&self, key: usize) -> Option<&mut T> {
        if let Some(capacity) = self.capacity.get() {
            if self.used > capacity || key > self.used {
                return None;
            }
        }

        if let Some(ptr) = self.ptr.as_ptr() {
            for i in 0..self.used {

                // Here we are doing element calculation on the raw pointer itself. The pointer
                // will automatically calculate how much to translate the pointer, so we just have
                // to give it the "index" that we want. We use this pointer like a good-old-C
                // version array.
                let real_ptr = unsafe { ptr.add(i) };
                let (raw_ptr, id) = unsafe { (&mut (*real_ptr).0, (*real_ptr).1) };

                if let Some(valid_id) = id {
                    if valid_id == key {
                        return Some(raw_ptr);
                    }
                }
            }
        }
        return None;
    }

    pub unsafe fn set_allocation(&mut self, buffer: &mut [u8], key_offset: usize) -> Result<(), QuantumError> {
        if self.ptr.is_valid() {
            return Err(QuantumError::AlreadySetValue);
        }

        let buffer_size = buffer.len();
        let self_size = self.get_bytes_per_element();
        let fitting_allocations = buffer_size / self_size;
        let is_enough_bytes = fitting_allocations > 0;

        if !is_enough_bytes {
            return Err(QuantumError::NoSpaceRemaining);
        }


        self.key_offset = key_offset;
        self.capacity = SafeSize::from_usize(fitting_allocations);
        self.used = 0;
        self.ptr = SafePtr::unsafe_from_address(VirtualAddress::from_ptr(buffer.as_ptr()));

        Ok(())
    }

    pub fn get_bytes_per_element(&self) -> usize {
        size_of::<(T, Option<usize>)>()
    }

    pub fn has_space_for_new_alloc(&self) -> bool {
        if let Some(capacity) = self.capacity.get() {
            if self.used >= capacity {
                return false;
            }
        } else {
            return false;
        }

        true
    }

    pub fn is_allocated(&self) -> bool {
        self.ptr.is_valid()
    }

    pub fn max_elements(&self) -> Option<usize> {
        self.capacity.get()
    }

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
    internal_buffer: Option<Vec<SafePtr<BufferComponent <T>>, 255>>,

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
    total_capacity: usize,

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
    total_allocations: usize,

    /// the percentage that we free `BufferComponent`'s
    to_free_percentage: usize,

    // the option to define if we should allocate automatically or not
    should_manually_allocate: bool,
}

impl<T> ResizeableBuffer<T> {
    pub fn new() -> Self {
        Self {
            internal_buffer: None,
            total_capacity: 0,
            used_elements: 0,
            total_allocations: 0,
            to_free_percentage: 60,
            should_manually_allocate: false
        }
    }

    fn init_to_zero(&mut self) {
        // Reset and Init the buffer components
        self.internal_buffer = Some(Vec::new());
        self.total_allocations = 0;
        self.total_capacity = 0;
        self.used_elements = 0;
        self.to_free_percentage = 60;
    }

    unsafe fn add_byte_array_to_free_component(&mut self, buffer: &mut [u8]) -> Result<(), QuantumError> {
        // make sure our buffer is defined
        if self.internal_buffer.is_none() {
            self.init_to_zero();
        }

        if let Some(internal_buffer) = &mut self.internal_buffer {

            // First check if the buffer is full
            let remaining_size = internal_buffer.capacity() - internal_buffer.len();
            if remaining_size == 0 {
                return Err(QuantumError::NoSpaceRemaining);
            }

            // Second check if we are pushing the first allocation
            if internal_buffer.len() == 0 {
                if !internal_buffer.is_full() {
                    if buffer.len() <= size_of::<BufferComponent<T>>() {
                        return Err(QuantumError::NoSpaceRemaining);
                    }

                    let new_component: BufferComponent<T> = BufferComponent::new();
                    let translated_buffer_type =
                        SafePtr::<BufferComponent<T>>::unsafe_from_address(VirtualAddress::new(buffer.as_ptr() as u64));

                    if let Some(ptr) = translated_buffer_type.as_ptr() {
                        *ptr = new_component;
                        let resl = internal_buffer.push(translated_buffer_type);

                        if resl.is_err() {
                            return Err(QuantumError::NoSpaceRemaining);
                        }
                    }

                } else {
                    return Err(QuantumError::BufferFull)
                }
            }

            // Finally add the allocation to the pool
            for i in internal_buffer {
                if let Some(ptr) = i.as_ptr() {
                    let buffer_comp = &mut *ptr;

                    let is_not_allocated = !buffer_comp.is_allocated();

                    if is_not_allocated {
                        let offset_size = size_of::<BufferComponent<T>>();
                        let buff = buffer.as_mut_ptr().add(offset_size);
                        let allocation_size = buffer.len() - offset_size;
                        let array = core::slice::from_raw_parts_mut(buff, allocation_size);

                        buffer_comp.set_allocation(array, self.total_capacity)?;

                        let our_size = size_of::<(T, usize)>();

                        let fitting_allocations = allocation_size / our_size;

                        self.total_allocations += 1;
                        self.total_capacity += fitting_allocations;

                        return Ok(());
                    }
                }
            }
            return Err(QuantumError::NoItem);
        }

        Err(QuantumError::UndefinedValue)
    }

    pub unsafe fn add_allocation(&mut self, bytes: &mut [u8]) -> Result<(), QuantumError> {
        // TODO: Ensure that the byte array does not go out of scope

        self.add_byte_array_to_free_component(bytes)?;

        Ok(())
    }

    fn get_buffer_component_with_key(&mut self, element_index: usize) -> Option<&mut BufferComponent<T>> {
        if let Some(vector) = &mut self.internal_buffer {
            for i in vector {
                if let Some(ptr) = i.as_ptr() {
                    let buffer_comp = unsafe { &mut *ptr };

                    if buffer_comp.does_contain_element(element_index) {
                        return Some(buffer_comp);
                    }
                }
            }
        }

        None
    }

    pub fn get_element(&mut self, index: usize) -> Option<&mut T> {
        if let Some(comp) = self.get_buffer_component_with_key(index) {
            if let Some(element) = comp.get_element_with_key(index) {
                return Some(element);
            }
        }

        None
    }

    pub fn set_element(&mut self, index: usize, element: T) -> Result<(), QuantumError> {
        if let Some(comp) = self.get_buffer_component_with_key(index) {
            if let Some(mut comp) = comp.get_element_with_key(index) {
                *comp = element;

                debug_println!("assigned existing element index {}", index);

                return Ok(());
            }
        }

        if let Some(vector) = &mut self.internal_buffer {
            for i in vector {
                if let Some(ptr) = i.as_ptr() {
                    let buffer_comp = unsafe { &mut *ptr };

                    if buffer_comp.has_space_for_new_alloc() {
                        buffer_comp.set_element(element, index)?;

                        self.used_elements += 1;

                        return Ok(());
                    }
                }
            }
        }

        Err(QuantumError::UndefinedValue)
    }

    pub fn push(&mut self, element: T) -> Result<(), QuantumError> {
        self.set_element(self.used_elements, element)
    }

    pub fn len(&self) -> usize {
        self.used_elements
    }

    pub fn total_free_allocations(&self) -> usize {
        self.total_capacity - self.used_elements
    }

    pub fn pop(&mut self, key: usize) -> Result<(), QuantumError> {
        Ok(())
    }
}

#[cfg(test)]
mod test_case {
    use core::mem::size_of;
    use crate::{debug_println, serial_print, serial_println};
    use crate::memory_utils::resizeable_buffer::{BufferComponent, ResizeableBuffer};
    use crate::test_handler::test_runner;

    #[test_case]
    fn buffer_component_setting_allocation() {
        let mut raw_vector_limited_lifetime = [0_u8; 4096];
        let mut test_component : BufferComponent<u8> = BufferComponent::new();

        unsafe {
            test_component.set_allocation(&mut raw_vector_limited_lifetime, 0)
                .expect("Unable to set raw bytes to BufferComponent");
        }
    }

    #[test_case]
    fn buffer_component_setting_allocation_with_different_type() {
        let mut raw_vector_limited_lifetime = [0_u8; 4096];
        let mut test_component : BufferComponent<u64> = BufferComponent::new();

        unsafe {
            test_component.set_allocation(&mut raw_vector_limited_lifetime, 0)
                .expect("Unable to set raw bytes to BufferComponent");
        }
    }

    #[test_case]
    fn buffer_component_adding_element_u8() {
        let mut raw_vector_limited_lifetime = [0_u8; 4096];
        let mut test_component : BufferComponent<u8> = BufferComponent::new();

        unsafe {
            test_component.set_allocation(&mut raw_vector_limited_lifetime, 0)
                .expect("Unable to set raw bytes to BufferComponent");
        }

        // Make sure the buffer isn't expanding because we should just be setting the
        // same element 255 times!
        for i in 0..255 {
            test_component.set_element(i, 0)
                .expect("Unable to push element into the BufferComponent");

            assert_eq!(*test_component.get_element_with_key(0).unwrap(), i);
            assert_eq!(test_component.used, 1);
        }

        serial_print!("  [SIZE: {}]  ", test_component.get_bytes_per_element());
        serial_print!("  [MAX: {}]  ", test_component.max_elements().unwrap());

        // Test if the buffer expands
        for i in 0..(test_component.max_elements().unwrap() as u8) {
            test_component.set_element(i, i as usize)
                .expect("Unable to push element into the Buffer");

            assert_eq!(*test_component.get_element_with_key(i as usize)
                .expect("Could not get element with that key!"), i);
            assert_eq!(test_component.used, i as usize + 1);
        }
    }

    #[test_case]
    fn buffer_component_adding_element_u64() {
        let mut raw_vector_limited_lifetime = [0_u8; 4096];
        let mut test_component : BufferComponent<u64> = BufferComponent::new();

        unsafe {
            test_component.set_allocation(&mut raw_vector_limited_lifetime, 0)
                .expect("Unable to set raw bytes to BufferComponent");
        }

        // Make sure the buffer isn't expanding because we should just be setting the
        // same element 255 times!
        for i in 0..255_u64 {
            test_component.set_element(i, 0)
                .expect("Unable to push element into the BufferComponent");

            assert_eq!(*test_component.get_element_with_key(0).unwrap(), i);
            assert_eq!(test_component.used, 1);
        }

        serial_print!("  [SIZE: {}]  ", test_component.get_bytes_per_element());
        serial_print!("  [MAX: {}]  ", test_component.max_elements().unwrap());

        // Test if the buffer expands
        for i in 0..(test_component.max_elements().unwrap() as u64) {
            test_component.set_element(i, i as usize)
                .expect("Unable to push element into the Buffer");

            assert_eq!(*test_component.get_element_with_key(i as usize)
                .expect("Could not get element with that key!"), i);
            assert_eq!(test_component.used, i as usize + 1);
        }
    }

    #[test_case]
    fn buffer_component_adding_element_many_bytes() {
        const AMOUNT_OF_BYTES_TO_TEST: usize = 1024;

        let mut raw_vector_limited_lifetime = [0_u8; 4096];
        let mut test_component : BufferComponent<[u8; AMOUNT_OF_BYTES_TO_TEST]> = BufferComponent::new();

        unsafe {
            test_component.set_allocation(&mut raw_vector_limited_lifetime, 0)
                .expect("Unable to set raw bytes to BufferComponent");
        }

        // Make sure the buffer isn't expanding because we should just be setting the
        // same element 255 times!
        for i in 0..255 {
            test_component.set_element([i; AMOUNT_OF_BYTES_TO_TEST], 0)
                .expect("Unable to push element into the BufferComponent");

            assert_eq!(*test_component.get_element_with_key(0).unwrap(), [i; AMOUNT_OF_BYTES_TO_TEST]);
            assert_eq!(test_component.used, 1);
        }

        serial_print!("  [SIZE: {}]  ", test_component.get_bytes_per_element());
        serial_print!("  [MAX: {}]  ", test_component.max_elements().unwrap());

        // Test if the buffer expands
        for i in 0..(test_component.max_elements().unwrap() as u8) {
            test_component.set_element([i; AMOUNT_OF_BYTES_TO_TEST], i as usize)
                .expect("Unable to push element into the Buffer");

            assert_eq!(test_component.does_contain_element(i as usize), true);
            assert_eq!(*test_component.get_element_with_key(i as usize)
                .expect("Could not get element with that key!"), [i; AMOUNT_OF_BYTES_TO_TEST]);
            assert_eq!(test_component.used, i as usize + 1);
        }
    }

    #[test_case]
    fn buffer_component_too_big_element() {
        const AMOUNT_OF_BYTES_TO_TEST: usize = 1024;

        let mut raw_vector_limited_lifetime = [0_u8; 10];
        let mut test_component : BufferComponent<[u8; AMOUNT_OF_BYTES_TO_TEST]> = BufferComponent::new();

        let result = unsafe {
            test_component.set_allocation(&mut raw_vector_limited_lifetime, 0)
        };
        assert_eq!(result.is_err(), true);

    }

    #[test_case]
    fn resizeable_buffer_setting_allocation() {
        let mut raw_vector_limited_lifetime = [0_u8; 4096];
        let mut vector: ResizeableBuffer<u8> = ResizeableBuffer::new();

        unsafe {
            vector.add_allocation(&mut raw_vector_limited_lifetime).unwrap();
        }

        for i in 0..50 {
            vector.push(i).expect("Unable to add element to Buffer");

            assert_eq!(vector.len(), i as usize + 1);
            assert_eq!(*vector.get_element(i as usize)
                           .expect("Unable to get element"), i);
        }

        for i in 0..50 {
            let element = *vector.get_element(i)
                .expect("Unable to get element!");

            assert_eq!(element - (i as u8), 0);
        }
    }

    #[test_case]
    fn get_total_buffer_component_overhead() {
        serial_print!(" [Overhead for u8: {}] ", size_of::<BufferComponent<u8>>());
    }

    #[test_case]
    fn get_total_buffer_overhead() {
        serial_print!(" [Overhead for u8: {}] ", size_of::<ResizeableBuffer<u8>>());
    }

    #[test_case]
    fn removing_and_adding_elements() {
        let mut raw_vector_limited_lifetime = [0_u8; 4096];
        let mut vector: ResizeableBuffer<u8> = ResizeableBuffer::new();

        unsafe {
            vector.add_allocation(&mut raw_vector_limited_lifetime).unwrap();
        }


    }


}