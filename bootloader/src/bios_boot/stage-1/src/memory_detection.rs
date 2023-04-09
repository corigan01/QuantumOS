/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
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

use crate::bios_println;
use crate::error::BootloaderError;
use core::arch::asm;
use quantum_lib::heapless_vector::HeaplessVec;

pub enum MemoryType {
    Usable,
    Reserved,
    AcpiReclaimableMemory,
    AcpiNVSMemory,
    BadMemoryRegion,
    Ignored,
    NonVolatile,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct BiosMemoryEntry {
    base_addr: u64,
    length: u64,
    memory_type: u32,
    acpi: u32,
}

pub struct MemoryMap {
    entries: HeaplessVec<BiosMemoryEntry, 32>,
}

impl MemoryMap {
    const MAGIC: u32 = 0x534D4150;

    pub fn new() -> Self {
        Self {
            entries: HeaplessVec::new(),
        }
    }

    pub fn quarry() -> Result<Self, BootloaderError> {
        let mut memory_map = Self::new();

        let asm_runner = |offset: &mut usize| {
            let mut memory_entry = BiosMemoryEntry::default();

            let mut status = 0;
            let mut written_data_amount = 0;

            let entry_ptr = &mut memory_entry as *mut BiosMemoryEntry as *mut u8;

            unsafe {
                asm!(
                    "push ebx",
                    "mov ebx, edx",
                    "mov edx, 0x534D4150",
                    "int 0x15",
                    "mov edx, ebx",
                    "pop ebx",
                    inout("eax") 0xe820 => status,
                    inout("edx") *offset,
                    inout("ecx") 24 => written_data_amount,
                    in("di") entry_ptr
                )
            };

            bios_println!("Memory entry {memory_entry:?}");

            if status != Self::MAGIC || written_data_amount == 0 {
                return Err(BootloaderError::BiosCallFailed);
            }

            if *offset == 0 {
                return Err(BootloaderError::NoValid);
            }

            Ok(memory_entry)
        };

        for i in 0..32 {
            bios_println!("attempting to get {i}' memory map!");
            let mut offset = i;
            let status = asm_runner(&mut offset);

            if status.contains_err(&BootloaderError::NoValid) {
                break;
            }

            memory_map.entries.push_within_capsity(status?);
        }

        Ok(memory_map)
    }
}
