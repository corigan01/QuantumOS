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
use core::ops::Range;
use crate::arch_x86_64::CpuPrivilegeLevel;
use crate::memory::VirtualAddress;
use crate::{serial_print, serial_println};

#[derive(Clone, Copy)]
#[repr(C, packed(2))]
pub struct IdtRegister {
    bytes: u16,
    ptr: VirtualAddress
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed(2))]
pub struct IdtEntry {
    offset1: u16,
    selector: u16,
    ist: u8,
    type_attributes: u8,
    offset2: u16,
    offset3: u32,
    zero: u32
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum TableSelect {
    GDT = 0,
    LDT = 1
}

#[derive(Clone, Copy, Debug)]
pub struct SegmentSelector {
    index: u16,
    table: TableSelect,
    dpl: CpuPrivilegeLevel
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum GateType {
    InterruptGate = 0xE,
    TrapGate = 0xF
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    table: [IdtEntry; 255]
}

impl IdtRegister {
    pub fn load(self) {
        serial_println!("\nIdtloading info ptr* {:p}, vaddr {:X}, size {:X}", &self, self.ptr.as_u64(), self.bytes);
        unsafe {
            asm!("lidt [{}]", in(reg) VirtualAddress::from_ptr(&self).as_u64(), options(readonly, nostack, preserves_flags));
        }
    }
}

impl SegmentSelector {
    pub fn new(index: u16, ti: TableSelect, dpl: CpuPrivilegeLevel) -> Self {
        SegmentSelector {
            index: SegmentSelector::try_index(index).unwrap(),
            table: ti,
            dpl
        }
    }

    pub fn try_index(index: u16) -> Result<u16, ()> {
        // checks if the index is valid
        match index < 0xE000 {
            false => Err(()),
            true  => Ok(index)
        }
    }

    pub fn as_u16(&self) -> u16 {
        (self.index << 3) | (((self.table as u8 & 0b1) as u16) << 2) | ((self.dpl as u8 & 0b11) as u16)
    }
}

impl IdtEntry {
    pub fn new() -> Self {
        IdtEntry {
            offset1: 0,
            selector: 0,
            ist: 0,
            type_attributes: 0,
            offset2: 0,
            offset3: 0,
            zero: 0 // must be 0x00
        }
    }


    pub fn set_type_attributes(&mut self, present: bool, dpl: CpuPrivilegeLevel, gate_type: GateType) {
        let present = present as u8;
        let dpl = dpl as u8;
        let gate_type = gate_type as u8;

        // Type Attributes table (part of IdtEntry)
        //
        //  |  47  | 46-45 | 44 |  43  -  40  |
        //  |======|=======|====|=============|
        //  |   P  |  DPL  | x0 |  Gate Type  |
        //  |======|=======|====|=============|
        //
        //             bits:         7              6-5          4            3-0
        self.type_attributes = (present << 7) | (dpl << 5) | (0x00 << 4) | gate_type;

        serial_println!("Type 0x{:X} 0b{:b}", self.type_attributes, self.type_attributes);
    }

    pub fn offset_as_u64(&self) -> u64 {
        let mut ret = 0;
        ret |= self.offset1 as u64;
        ret |= (self.offset2 as u64) << 16;
        ret |= (self.offset3 as u64) << 32;
        ret
    }

    pub fn set_segment_selector(&mut self, segment: SegmentSelector) {
        serial_println!("Segment 0x{:x}", segment.as_u16());

        self.selector = segment.as_u16();
    }

    pub fn set_gate(&mut self, f: VirtualAddress) {
        let pointer = f.as_u64();

        self.offset1 = pointer as u16;
        self.offset2 = (pointer >> 0x10) as u16;
        self.offset3 = (pointer >> 0x20) as u32;
    }
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        InterruptDescriptorTable {
            table: [IdtEntry::new(); 255],
        }
    }
    
    pub fn add_entry_range(&mut self, entry: IdtEntry, range: Range<u8>) -> Result<(), &str> {
        if let Some(max) = range.clone().max() {
            if max >= 255 {
                return Err("Range bound is invalid, Range should only be 0-254");
            }
        }

        for i in range {
            let res = Self::add_entry(self, entry, i as usize);

            if res.is_err() {
                return Err("Failed to add all entries")
            }
        }

        Ok(())
    }

    pub fn add_entry(&mut self, entry: IdtEntry, index: usize) -> Result<(), &str> {
        let is_addr_null = entry.offset_as_u64() == 0x0;
        if index >= 255 { return Err("Index out of bounds"); }
        if is_addr_null && entry.type_attributes & 0x80 != 0 { return Err("Present bit is set, but no address to call"); }

        self.table[index] = entry;

        Ok(())
    }

    pub fn remove_entry(&mut self, index: usize) -> Result<(), &str> {
        if index >= 255 { return Err("Index out of bounds"); }
        if self.table[index].type_attributes & 0x80 != 0 { return Err("Can't remove a non-present entry"); }

        self.table[index] = IdtEntry::new();

        Ok(())
    }

    pub fn submit_entries(&'static self) -> IdtRegister {
        use core::mem::size_of;
        IdtRegister {
            bytes: (size_of::<Self>() - 1) as u16,
            ptr: VirtualAddress::from_ptr(&self)
        }
    }
}

