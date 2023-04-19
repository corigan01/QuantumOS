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

use core::marker::PhantomData;
use core::mem::transmute;
use crate::address_utils::virtual_address::{Aligned, VirtAddress};
use crate::x86_64::paging::structures::{PageMapLevel1, PageMapLevel2, PageMapLevel3, PageMapLevel4};

pub struct NonTypedPageConfig;

pub trait PageConfigable {}
impl PageConfigable for PageMapLevel1 {}
impl PageConfigable for PageMapLevel2 {}
impl PageConfigable for PageMapLevel3 {}
impl PageConfigable for PageMapLevel4 {}

pub struct PageConfigBuilder<Type = NonTypedPageConfig> {
    pressent: bool,
    rw: bool,
    user_superviser: bool,
    writh_through: bool,
    cache_disable: bool,
    accessed: bool,
    page_size_select: bool,
    available: [u8; 6],
    maximum: bool,
    physical_address_bit: bool,
    execute_disable: bool,
    dirty: bool,
    protection_key: [u8; 3],
    page_attribute_table: bool,
    global: bool,
    address: u64,
    reserved: PhantomData<Type>
}

impl PageConfigBuilder {
    pub fn new() -> Self {
        Self {
            pressent: false,
            rw: false,
            user_superviser: false,
            writh_through: false,
            cache_disable: false,
            accessed: false,
            page_size_select: false,
            available: [0; 6],
            maximum: false,
            physical_address_bit: false,
            execute_disable: false,
            dirty: false,
            protection_key: [0; 3],
            page_attribute_table: false,
            global: false,
            address: 0,
            reserved: Default::default(),
        }
    }
}

impl PageConfigBuilder<NonTypedPageConfig> {
    pub fn level1(self) -> PageConfigBuilder<PageMapLevel1> {
        unsafe {
            transmute(self)
        }
    }
    
    pub fn level2(self) -> PageConfigBuilder<PageMapLevel2> {
        unsafe {
            transmute(self)
        }
    }
    
    pub fn level3(self) -> PageConfigBuilder<PageMapLevel3> {
        unsafe {
            transmute(self)
        }
    }
    
    pub fn level4(self) -> PageConfigBuilder<PageMapLevel4> {
        unsafe {
            transmute(self)
        }
    }
}

impl<Type> PageConfigBuilder<Type>
    where Type: PageConfigable {

    pub fn present(mut self, flag: bool) -> Self {
        self.pressent = flag;

        self
    }

    pub fn executable(mut self, flag: bool) -> Self {
        self.execute_disable = !flag;

        self
    }

    pub fn read_write(mut self, flag: bool) -> Self {
        self.rw = flag;

        self
    }

    pub fn cache_disable(mut self, flag: bool) -> Self {
        self.cache_disable = flag;

        self
    }

    pub fn write_through(mut self, flag: bool) -> Self {
        self.writh_through = flag;

        self
    }

    pub fn user_page(mut self, flag: bool) -> Self {
        self.user_superviser = flag;

        self
    }
}

impl PageConfigBuilder<PageMapLevel1> {
    pub fn set_address(mut self, address: VirtAddress<Aligned, 12>) -> Self {
        self.address = address.try_into().unwrap();

        self
    }
}


