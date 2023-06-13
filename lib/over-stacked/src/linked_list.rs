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

// This is really nasty code, dont look :)

use core::ptr::NonNull;
use quantum_utils::own_ptr::OwnPtr;

pub struct LinkedListComponentIter<'a, Type>
    where Type: ?Sized {
    linked_list: &'a LinkedListComponent<Type>,
    index: usize
}

impl<'a, Type> Iterator for LinkedListComponentIter<'a, Type>
    where Type: Copy + ?Sized {

    type Item = &'a Type;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.linked_list.get_nth_ref(self.index - 1)
    }

}

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

    pub fn next_list_ref(&self) -> Option<&Self> {
        Some(unsafe { self.next_element_ptr?.as_ref() })
    }

    pub fn next_list_mut(&mut self) -> Option<&mut Self> {
        Some(unsafe { self.next_element_ptr?.as_mut() })
    }

    pub fn last_list_ref(&self) -> &Self {
        let mut next =
            if let Some(value) = self.next_list_ref() {
                value
            } else {
                return self;
            };

        loop {
            if let Some(value) = next.next_list_ref() {
                next = value;

                continue;
            }

            break next;
        }
    }

    pub fn last_list_mut(&mut self) -> &mut Self {
        let mut next =
            if let Some(value) = self.next_list_ref() {
                value
            } else {
                return self;
            };

        loop {
            if let Some(value) = next.next_list_ref() {
                next = value;

                continue;
            }

            // TODO: This is a hack to get around the stupid `cant borrow mut more then once` problem!
            break unsafe { &mut *(next as *const Self as *mut Self) };
        }
    }

    pub fn next_ref(&self) -> Option<&Type> {
        let next = self.next_list_ref()?;

        Some(next.as_ref())
    }

    pub fn next_mut(&mut self) -> Option<&mut Type> {
        let next = self.next_list_mut()?;

        Some(next.as_mut())
    }

    pub fn self_ptr(&mut self) -> NonNull<Self> {
        let self_ptr = self as *mut Self;

        unsafe { NonNull::new_unchecked(self_ptr) }
    }

    pub fn get_nth_ref(&self, n: usize) -> Option<&Type> {
        if n == 0 {
            return Some(self.as_ref());
        }

        let mut next = self.next_list_ref()?;
        for _i in 0..(n - 1) {
            next = next.next_list_ref()?;
        }

        Some(next.as_ref())
    }

    pub fn get_nth_mut(&mut self, n: usize) -> Option<&mut Type> {
        if n == 0 {
            return Some(self.as_mut());
        }

        let mut next = self.next_list_mut()?;
        for _i in 0..(n - 1) {
            next = next.next_list_mut()?;
        }

        Some(next.as_mut())
    }

    pub fn iter(&self) -> LinkedListComponentIter<Type> {
        LinkedListComponentIter {
            linked_list: self,
            index: 0,
        }
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

        assert!(linked_list_main.next_element_ptr.is_none());

        linked_list_main.recurse_next_element(linked_list_two.self_ptr());

        assert!(linked_list_main.next_element_ptr.is_some());

        assert_eq!(linked_list_main.as_ref(), &1);
        assert_eq!(linked_list_main.next_list_ref().unwrap().as_ref(), &2);
    }

    #[test]
    fn test_getting_next_element() {
        let mut data_one = 1;
        let mut data_two = 2;
        let mut data_three = 3;

        let own_ptr_one = OwnPtr::from_mut(&mut data_one);
        let own_ptr_two = OwnPtr::from_mut(&mut data_two);
        let own_ptr_three = OwnPtr::from_mut(&mut data_three);

        let mut main_linked_list = LinkedListComponent::new(own_ptr_one);

        let mut linked_two = LinkedListComponent::new(own_ptr_two);
        let mut linked_three = LinkedListComponent::new(own_ptr_three);

        main_linked_list.recurse_next_element(linked_two.self_ptr());

        assert!(matches!(main_linked_list.next_list_mut(), Some(_)));

        // Test getting the second, and then try to add to it
        main_linked_list.next_list_mut().unwrap().recurse_next_element(linked_three.self_ptr());

        assert_eq!(main_linked_list.as_ref(), &1);
        assert_eq!(main_linked_list.next_ref().unwrap(), &2);
        assert_eq!(main_linked_list.next_list_ref().unwrap().next_ref().unwrap(), &3);
    }

    #[test]
    fn iterator_over_linked_list() {
        let mut data_one = 1;
        let mut data_two = 2;
        let mut data_three = 3;

        let own_ptr_one = OwnPtr::from_mut(&mut data_one);
        let own_ptr_two = OwnPtr::from_mut(&mut data_two);
        let own_ptr_three = OwnPtr::from_mut(&mut data_three);

        let mut main_linked_list = LinkedListComponent::new(own_ptr_one);

        let mut linked_two = LinkedListComponent::new(own_ptr_two);
        let mut linked_three = LinkedListComponent::new(own_ptr_three);

        main_linked_list.recurse_next_element(linked_two.self_ptr());

        assert!(matches!(main_linked_list.next_list_mut(), Some(_)));

        main_linked_list.last_list_mut().recurse_next_element(linked_three.self_ptr());

        assert!(matches!(linked_two.next_list_mut(), Some(_)));

        let mut linked_list_iter = main_linked_list.iter();

        assert_eq!(linked_list_iter.next(), Some(&1));
        assert_eq!(linked_list_iter.next(), Some(&2));
        assert_eq!(linked_list_iter.next(), Some(&3));
        assert_eq!(linked_list_iter.next(), None);

    }

}

