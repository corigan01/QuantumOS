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


use core::arch::asm;
use crate::arch_x86_64::tables;
use crate::memory::VirtualAddress;
use crate::serial_println;

#[derive(Clone, Copy)]
#[repr(C, packed(2))]
pub struct GdtRegister {
    size: u16,
    ptr: VirtualAddress
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed(2))]
pub struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8
}

#[derive(Clone, Copy, Debug)]
pub struct GlobalDescriptorTable {
    table: [GdtEntry; 8],
    length: u16
}

impl GdtRegister {

    #[inline]
    pub fn load(self) {
        unsafe {
            asm!("lgdt [{}]", in(reg) VirtualAddress::from_ptr(&self).as_u64(), options(readonly, nostack, preserves_flags));
        }
    }
}

impl GdtEntry {
    pub fn new() -> Self {
        GdtEntry {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0,
            granularity: 0,
            base_high: 0
        }
    }

    pub fn new_raw(base: u32, limit: u32, access: u8, gran: u8) -> Self {
        let mut entry = GdtEntry::new();

        entry.base_low = (base & 0xFFFF) as u16;
        entry.base_middle = ((base >> 16) & 0xFF) as u8;
        entry.base_high = ((base >> 24) & 0xFF) as u8;

        entry.limit_low = (limit & 0xFFFF) as u16;
        entry.granularity = ((limit >> 16) & 0x0F) as u8;

        entry.granularity |= (gran & 0xF0) as u8;
        entry.access = access;

        entry
    }

    pub fn as_u64(&self) -> u64 {
        (self.limit_low as u64) | ((self.base_low as u64) << 0x10_u64) |
            ((self.base_middle as u64) << 0x18_u64) | ((self.access as u64) << 0x20_u64) |
            ((self.granularity as u64) << 0x28_u64) | ((self.base_high as u64) << 0x30_u64)
    }

    pub fn set_null(&mut self) {
        *self = GdtEntry::new();
    }

}

impl GlobalDescriptorTable {
    pub fn new() -> Self {
        GlobalDescriptorTable {
            table: [GdtEntry::new(); 8],
            length: 0
        }
    }

    pub fn add_entry(&mut self, entry: GdtEntry) -> Result<(), &str> {
        if self.length >= 8 { return Err("Too many entries in the GDT"); }
        if self.length == 0 && entry.as_u64() != 0x00 {return Err("First entry must be null"); }

        self.table[self.length as usize] = entry;
        self.length += 1;

        return Ok(());
    }

    #[inline]
    pub fn submit_entries(&'static self) -> GdtRegister {
        GdtRegister {
            size: (self.length * 8) - 1,
            ptr: VirtualAddress::from_ptr(self.table.as_ptr())
        }
    }
}

/*
unsafe fn load_gdt_ptr() {
    asm!("lgdt [{}]", in(reg) gdt, options(readonly, nostack, preserves_flags));
}*/