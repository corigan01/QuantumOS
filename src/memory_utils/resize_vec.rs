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

use core::marker::PhantomData;
use core::mem::size_of;
use core::ptr;
use crate::{debug_println, debug_print};
use crate::error_utils::QuantumError;
use crate::memory::VirtualAddress;
use crate::memory_utils::safe_ptr::SafePtr;

#[derive(Debug)]
struct RecursiveComponent<'a, T> {
    ptr: &'a mut [u8],
    ph: PhantomData<T>
}

struct ComponentInformation<'a,  T> {
    data_ptr: *const T,
    total: usize,
    used: usize,
    next_ptr: *const RecursiveComponent<'a, T>
}

// Data stored in beginning of ptr
// (Next Ptr) (Used) [DATA] (END FILLER)
// Overhead is as minimal as possible

impl<'a, T> RecursiveComponent<'a, T> {
    pub fn new(bytes: &'a mut [u8]) -> Result<Self, QuantumError> {
        let total_bytes_size = bytes.len();
        let size_of_overhead = size_of::<(Self, usize)>();
        let data_section_size = total_bytes_size - size_of_overhead;

        if total_bytes_size <= size_of_overhead && data_section_size <= size_of::<T>() {
            return Err(QuantumError::NoSpaceRemaining);
        }

        let raw_new = Self {
            ptr: bytes,
            ph: PhantomData::default()
        };

        Ok(raw_new)
    }

    pub fn get_buffer_info(&mut self) -> ComponentInformation<T> {
        let ptr = self.ptr.as_mut_ptr() as *mut u8;
        let total_size_of_buffer= self.ptr.len();
        let size_of_over_head = size_of::<(Self, usize)>();

        let size_of_data_section = total_size_of_buffer - size_of_over_head;
        let total_fitting_allocations = size_of_data_section / size_of::<T>();

        let next_vector_ptr = ptr as *mut Self;

        let info_ptr = ptr as *mut u64;
        let used_data = unsafe { *(info_ptr.add(1)) };

        let shifted_ptr = unsafe { info_ptr.add(2) };
        let data_ptr = shifted_ptr as *mut T;

        ComponentInformation::<T> {
            data_ptr,
            total: total_fitting_allocations,
            used: used_data as usize,
            next_ptr: next_vector_ptr
        }
    }

    fn modify_used(&mut self, modify: isize) {
        let ptr = self.ptr.as_mut_ptr() as *mut u64;
        let mut size_ref = unsafe  { &mut *(ptr.add(1)) };
        *size_ref = (*size_ref as isize + modify) as u64;
    }

    pub fn push(&mut self, element: T) -> Result<(), QuantumError> {
        let self_info = self.get_buffer_info();
        let mut data_ptr = self_info.data_ptr as *mut T;

        // check if new data fits
        if self_info.total <= self_info.used {
            return Err(QuantumError::BufferFull);
        }

        // add the data to the buffer
        unsafe { *(data_ptr.add(self_info.used)) = element };
        self.modify_used(1);

        Ok(())
    }

    pub fn get(&mut self, key: usize) -> Result<&mut T, QuantumError> {
        let self_info = self.get_buffer_info();

        if key > self_info.used {
            return Err(QuantumError::NoItem);
        }

        let mut data_ptr = self_info.data_ptr as *mut T;

        let value = unsafe {
            &mut *data_ptr.add(key)
        };

        Ok(value)
    }

    pub fn remove_element(&mut self, key: usize) {
        let self_info = self.get_buffer_info();
        let data_ptr = self_info.data_ptr;

        for i in (key + 1)..self_info.used {
            let prev_index = i - 1;
            let mut prev_ptr = unsafe { data_ptr.add(prev_index) as *mut T };
            let mut current_ptr = unsafe { data_ptr.add(i) as *mut T };

            unsafe {
                *prev_ptr = core::ptr::read(current_ptr);
            }
        }

        self.modify_used(-1);
    }

    pub fn len(&mut self) -> usize {
        self.get_buffer_info().used
    }

    pub fn total_size(&mut self) -> usize {
        self.get_buffer_info().total
    }

    pub fn is_full(&mut self) -> bool {
        let info = self.get_buffer_info();

        info.used >= info.total
    }

    pub fn recurse_next_component(&mut self, component: Self) -> Result<(), QuantumError> {
        if self.is_parent() {
            return Err(QuantumError::ExistingValue);
        }

        let mut self_ptr = self.ptr.as_mut_ptr() as *mut Self;
        unsafe { *self_ptr = component  };

        Ok(())
    }

    pub fn is_parent(&mut self) -> bool {
        let mut next_comp = self.get_buffer_info().next_ptr as *mut Self;
        let next_info = unsafe { &mut *next_comp };
        next_info.ptr.as_ptr() as u64 > 0
    }

    pub fn get_child(&mut self) -> Option<&mut Self> {
        if self.is_parent() {
            let mut next_comp = self.get_buffer_info().next_ptr as *mut Self;
            let mut child = unsafe { &mut *next_comp };


            return Some(child);
        }

        None
    }
}

pub struct ByteVec<'a, T> {
    parent: Option<RecursiveComponent<'a, T>>
}

impl<'a, T> ByteVec<'a, T> {
    pub fn new() -> Self {
        Self {
            parent: None
        }
    }

    pub fn add_bytes(&mut self, bytes: &'a mut [u8]) -> Result<(), QuantumError> {
        if self.parent.is_none() {
            let mut component = RecursiveComponent::<T>::new(bytes)?;
            self.parent = Some(component);

            return Ok(());
        }

        if let Some(comp) = &mut self.parent {
            let mut parent = comp;
            loop {
                // Loop until we find an element without children
                if let Some(child) = parent.get_child() {
                    parent = child;

                } else {
                    break;
                }
            }

            // finally we found a parent without a child, so lets add one
            let child = RecursiveComponent::<T>::new(bytes)?;

            parent.recurse_next_component(child)?;

            return Ok(());
        }

        Err(QuantumError::UndefinedValue)
    }
}

#[cfg(test)]
mod test {
    use crate::memory_utils::resize_vec::RecursiveComponent;

    #[test_case]
    fn test_constructing_component() {
        let mut limited_lifetime_value = [0_u8; 4096];
        let component =
            RecursiveComponent::<u8>::new(&mut limited_lifetime_value)
                .expect("Could not construct vector!");
    }

    #[test_case]
    fn test_pushing_to_component() {
        let mut limited_lifetime_value = [0_u8; 4096];
        let mut component =
            RecursiveComponent::<u8>::new(&mut limited_lifetime_value)
                .expect("Could not construct vector!");

        component.push(10).expect("Could not push back value");
        assert_eq!(*component.get(0).unwrap(), 10_u8);
    }

    #[test_case]
    fn test_pushing_many_elements() {
        let mut limited_lifetime_value = [0_u8; 4096];
        let mut component =
            RecursiveComponent::<u8>::new(&mut limited_lifetime_value)
                .expect("Could not construct ");

        for i in 0..component.total_size() {
            component.push(i as u8).expect("Unable to push element");
            assert_eq!(*component.get(i).unwrap(), i as u8);
            assert_eq!(component.len(), i + 1);
        }

        for i in 0..component.total_size() {
            assert_eq!(*component.get(i).unwrap(), i as u8);
        }
    }

    #[test_case]
    fn test_push_and_remove_elements() {
        let mut limited_lifetime_value = [0_u8; 4096];
        let mut component =
            RecursiveComponent::<u8>::new(&mut limited_lifetime_value)
                .expect("Could not construct ");


        for i in 0..10 {
            component.push(i as u8).expect("Unable to push element!");
        }

        for i in 0..6 {
            component.remove_element(i);
        }

        for i in 0..5 {
            let check = (i * 2) + 1;

            assert_eq!(*component.get(i).unwrap(), check as u8);
        }

        assert_eq!(component.len(), 4);
    }

    #[test_case]
    fn test_different_sized_element() {
        let mut limited_lifetime_value = [0_u8; 4096];
        let mut component =
            RecursiveComponent::<u64>::new(&mut limited_lifetime_value)
                .expect("Could not construct ");


        for i in 0..component.total_size() {
            component.push(i as u64).expect("Unable to push element");

            assert_eq!(*component.get(i).unwrap(), i as u64);
        }

        assert_eq!(component.is_full(), true);
    }

    #[test_case]
    fn test_recursive_element() {
        let mut limited_lifetime_value = [0_u8; 4096];
        let mut component =
            RecursiveComponent::<u64>::new(&mut limited_lifetime_value)
                .expect("Could not construct ");

        let mut child_buffer = [0_u8; 4096];
        let mut child =
            RecursiveComponent::<u64>::new(&mut child_buffer)
                .expect("Could not construct ");

        assert_eq!(component.is_parent(), false);

        component.recurse_next_component(child).unwrap();

        assert_eq!(component.is_parent(), true);

        let test_child = component.get_child().unwrap();

        test_child.push(123).unwrap();
        assert_eq!(*test_child.get(0).unwrap(), 123);
    }



}