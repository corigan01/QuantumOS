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
use quantum_utils::own_ptr::OwnPtr;

pub struct LinkedListComponent<Type: ?Sized> {
    own_ptr: OwnPtr<Type>,
    next_element_ptr: Option<NonNull<Self>>
}

impl<Type: ?Sized> LinkedListComponent<Type> {
    pub fn new(value: OwnPtr<Type>) -> Self {
        Self {
            own_ptr: value,
            next_element_ptr: None
        }
    }

    pub fn store(&mut self, value: OwnPtr<Type>) {
        self.own_ptr = value;
    }

    pub fn release(self) -> OwnPtr<Type> {
        self.own_ptr
    }

    pub fn as_ref(&self) -> &Type {
        unsafe { self.own_ptr.as_ref() }
    }

    pub fn as_mut(&mut self) -> &mut Type {
        unsafe { self.own_ptr.as_mut() }
    }

    pub fn recurse_next_element(&mut self, next_element: NonNull<Self>) {
        self.next_element_ptr = Some(next_element);
    }

    pub fn get_next_list_ref(&self) -> Option<&Self> {
        Some(unsafe { self.next_element_ptr?.as_ref() })
    }

    pub fn next_ref(&self) -> Option<&Self> {
        Some(unsafe { self.next_element_ptr?.as_ref() })
    }

    pub fn next_mut(&mut self) -> Option<&mut Self> {
        Some(unsafe { self.next_element_ptr?.as_mut() })
    }

    pub fn self_ptr(&mut self) -> NonNull<Self> {
        let self_ptr = self as *mut Self;

        unsafe { NonNull::new_unchecked(self_ptr) }
    }
}

#[cfg(test)]
mod test {
    use quantum_utils::own_ptr::OwnPtr;
    use crate::linked_list::LinkedListComponent;

    #[test]
    fn test_storing_one_value() {
        let mut data_is_here = 102;
        let own_ptr_to_data = OwnPtr::from_mut(&mut data_is_here);

        let linked_list = LinkedListComponent::new(own_ptr_to_data);

        assert_eq!(linked_list.as_ref(), &102);
    }

    #[test]
    fn test_storing_two_values() {
        let mut data_one_is_here = 1;
        let mut data_two_is_here = 2;

        let own_ptr_one = OwnPtr::from_mut(&mut data_one_is_here);
        let own_ptr_two = OwnPtr::from_mut(&mut data_two_is_here);

        let mut linked_list_main = LinkedListComponent::new(own_ptr_one);
        let mut linked_list_two = LinkedListComponent::new(own_ptr_two);

        linked_list_main.recurse_next_element(linked_list_two.self_ptr());

        assert!(linked_list_main.next_element_ptr.is_some());

        assert_eq!(linked_list_main.as_ref(), &1);
        assert_eq!(linked_list_main.next_ref().unwrap().as_ref(), &2);
    }

}