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

use bootloader::e820_memory::E820Entry;
use quantum_lib::x86_64::bios_call::BiosCall;
use quantum_lib::x86_64::bios_call::BiosCallResult::Success;

pub fn get_memory_map(memory_map_region: &mut [E820Entry]) -> usize {
    let mut region_offset = 0;
    let mut last_entry_value = 0;

    loop {
        let mut entry = E820Entry::default();
        entry.len = 1;

        let ptr = &entry as *const E820Entry as *const u8;

        let value = unsafe {
            BiosCall::new().bit32_call().memory_detection_operating(ptr, last_entry_value)
        };

        if let Success(value) = value {
            memory_map_region[region_offset] = entry;
            last_entry_value = value;
            region_offset += 1;

            if value == 0 || region_offset >= memory_map_region.len()  {
                break;
            }
        } else {
            break;
        }
    }

    region_offset
}