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

use crate::address_utils::virtual_address::{Aligned, VirtAddress};
use crate::address_utils::VIRTUAL_ALLOWED_ADDRESS_SIZE;
use crate::bitset::BitSet;
use crate::x86_64::paging::structures::{
    PageMapLevel1Entry, PageMapLevel2Entry, PageMapLevel3Entry, PageMapLevel4Entry,
};
use crate::x86_64::paging::PagingErr;
use core::marker::PhantomData;
use core::mem::transmute;

pub struct NonTypedPageConfig;

pub trait PageConfigable {}
impl PageConfigable for PageMapLevel1Entry {}
impl PageConfigable for PageMapLevel2Entry {}
impl PageConfigable for PageMapLevel3Entry {}
impl PageConfigable for PageMapLevel4Entry {}

#[allow(dead_code)]
pub struct PageConfigBuilder<Type = NonTypedPageConfig> {
    present: bool,
    rw: bool,
    user_supervisor: bool,
    write_through: bool,
    cache_disable: bool,
    accessed: bool,
    page_size_select: bool,
    available: [u8; 6],
    maximum: bool,
    physical_address_bit: bool,
    execute_disable: bool,
    dirty: bool,
    protection_key: u8,
    page_attribute_table: bool,
    global: bool,
    address: u64,
    reserved: PhantomData<Type>,
}

impl PageConfigBuilder {
    pub fn new() -> Self {
        Self {
            present: false,
            rw: false,
            user_supervisor: false,
            write_through: false,
            cache_disable: false,
            accessed: false,
            page_size_select: false,
            available: [0; 6],
            maximum: false,
            physical_address_bit: false,
            execute_disable: false,
            dirty: false,
            protection_key: 0,
            page_attribute_table: false,
            global: false,
            address: 0,
            reserved: Default::default(),
        }
    }
}

impl PageConfigBuilder<NonTypedPageConfig> {
    pub fn level1(self) -> PageConfigBuilder<PageMapLevel1Entry> {
        unsafe { transmute(self) }
    }
    pub fn level2(self) -> PageConfigBuilder<PageMapLevel2Entry> {
        unsafe { transmute(self) }
    }
    pub fn level3(self) -> PageConfigBuilder<PageMapLevel3Entry> {
        unsafe { transmute(self) }
    }
    pub fn level4(self) -> PageConfigBuilder<PageMapLevel4Entry> {
        unsafe { transmute(self) }
    }
}

impl<Type> PageConfigBuilder<Type>
where
    Type: PageConfigable,
{
    pub fn present(mut self, flag: bool) -> Self {
        self.present = flag;

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
        self.write_through = flag;

        self
    }

    pub fn user_page(mut self, flag: bool) -> Self {
        self.user_supervisor = flag;

        self
    }
}

impl PageConfigBuilder<PageMapLevel1Entry> {
    pub fn set_address(mut self, address: VirtAddress<Aligned, 12>) -> Self {
        self.address = address.try_into().unwrap();

        self
    }

    pub fn build(self) -> Result<PageMapLevel1Entry, PagingErr> {
        // TODO: Add checking for invalid configs

        let mut compiled_options = 0_u64;

        compiled_options.set_bit(0, self.present);
        compiled_options.set_bit(1, self.rw);
        compiled_options.set_bit(2, self.user_supervisor);
        compiled_options.set_bit(3, self.write_through);
        compiled_options.set_bit(4, self.cache_disable);
        compiled_options.set_bit(5, self.accessed);
        compiled_options.set_bit(6, self.dirty);
        compiled_options.set_bit(7, self.page_attribute_table);
        compiled_options.set_bit(8, self.global);
        compiled_options.set_bits(12..(VIRTUAL_ALLOWED_ADDRESS_SIZE), self.address >> 12);
        compiled_options.set_bits(59..(62 + 1), self.protection_key as u64);
        compiled_options.set_bit(63, self.execute_disable);

        Ok(PageMapLevel1Entry::add_config_options_from_u64(
            compiled_options,
        ))
    }
}

impl PageConfigBuilder<PageMapLevel2Entry> {
    pub fn set_address_of_next_table(mut self, address: VirtAddress<Aligned, 12>) -> Self {
        self.address = address.try_into().unwrap();
        self.page_size_select = false;

        self
    }

    pub fn set_huge_page_address(mut self, address: VirtAddress<Aligned, 21>) -> Self {
        self.address = address.try_into().unwrap();
        self.page_size_select = true;

        self
    }

    pub fn build(self) -> Result<PageMapLevel2Entry, PagingErr> {
        // TODO: Add checking for invalid configs

        let mut compiled_options = 0_u64;

        compiled_options.set_bit(0, self.present);
        compiled_options.set_bit(1, self.rw);
        compiled_options.set_bit(2, self.user_supervisor);
        compiled_options.set_bit(3, self.write_through);
        compiled_options.set_bit(4, self.cache_disable);
        compiled_options.set_bit(5, self.accessed);
        compiled_options.set_bit(7, self.page_size_select);

        if self.page_size_select {
            compiled_options.set_bit(6, self.dirty);
            compiled_options.set_bit(8, self.global);
            compiled_options.set_bit(12, self.page_attribute_table);
            compiled_options.set_bits(59..(62 + 1), self.protection_key as u64);
            compiled_options.set_bits(21..(VIRTUAL_ALLOWED_ADDRESS_SIZE), self.address >> 21);
        } else {
            compiled_options.set_bits(12..(VIRTUAL_ALLOWED_ADDRESS_SIZE), self.address >> 12);
        }

        compiled_options.set_bit(63, self.execute_disable);

        Ok(PageMapLevel2Entry::add_config_options_from_u64(
            compiled_options,
        ))
    }
}

impl PageConfigBuilder<PageMapLevel3Entry> {
    pub fn set_address_of_next_table(mut self, address: VirtAddress<Aligned, 12>) -> Self {
        self.address = address.try_into().unwrap();
        self.page_size_select = false;

        self
    }

    pub fn set_huge_page_address(mut self, address: VirtAddress<Aligned, 30>) -> Self {
        self.address = address.try_into().unwrap();
        self.page_size_select = true;

        self
    }

    pub fn build(self) -> Result<PageMapLevel3Entry, PagingErr> {
        // TODO: Add checking for invalid configs

        let mut compiled_options = 0_u64;

        compiled_options.set_bit(0, self.present);
        compiled_options.set_bit(1, self.rw);
        compiled_options.set_bit(2, self.user_supervisor);
        compiled_options.set_bit(3, self.write_through);
        compiled_options.set_bit(4, self.cache_disable);
        compiled_options.set_bit(5, self.accessed);
        compiled_options.set_bit(7, self.page_size_select);

        if self.page_size_select {
            compiled_options.set_bit(6, self.dirty);
            compiled_options.set_bit(8, self.global);
            compiled_options.set_bit(12, self.page_attribute_table);
            compiled_options.set_bits(59..(62 + 1), self.protection_key as u64);
            compiled_options.set_bits(30..(VIRTUAL_ALLOWED_ADDRESS_SIZE), self.address >> 30);
        } else {
            compiled_options.set_bits(12..(VIRTUAL_ALLOWED_ADDRESS_SIZE), self.address >> 12);
        }

        compiled_options.set_bit(63, self.execute_disable);

        Ok(PageMapLevel3Entry::add_config_options_from_u64(
            compiled_options,
        ))
    }
}

impl PageConfigBuilder<PageMapLevel4Entry> {
    pub fn set_address_of_next_table(mut self, address: VirtAddress<Aligned, 12>) -> Self {
        self.address = address.try_into().unwrap();

        self
    }

    pub fn build(self) -> Result<PageMapLevel4Entry, PagingErr> {
        // TODO: Add checking for invalid configs

        let mut compiled_options = 0_u64;

        compiled_options.set_bit(0, self.present);
        compiled_options.set_bit(1, self.rw);
        compiled_options.set_bit(2, self.user_supervisor);
        compiled_options.set_bit(3, self.write_through);
        compiled_options.set_bit(4, self.cache_disable);
        compiled_options.set_bit(5, self.accessed);
        compiled_options.set_bits(12..(VIRTUAL_ALLOWED_ADDRESS_SIZE), self.address >> 12);
        compiled_options.set_bit(63, self.execute_disable);

        Ok(PageMapLevel4Entry::add_config_options_from_u64(
            compiled_options,
        ))
    }
}
