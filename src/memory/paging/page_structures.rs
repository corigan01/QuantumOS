/*!
```text
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
```
*/

use crate::debug_println;
use crate::error_utils::QuantumError;
use crate::memory::paging::page_flags::{PageFlagOptions, PageFlags};
use crate::memory::VirtualAddress;

#[repr(align(4096))]
struct PageMapLevel4 {
    entries: [PageFlags; 512]
}

#[repr(align(4096))]
struct PageDirPointerTable {
    entries: [PageFlags; 512]
}

#[repr(align(4096))]
struct PageDir {
    entries: [PageFlags; 512]
}

#[repr(align(4096))]
struct PageTable {
    entries: [PageFlags; 512]
}

macro_rules! map_impl {
    ($($t:ty)*) => ($(
        impl $t {
             pub fn new() -> Self {
                Self {
                    entries: [PageFlags::new(); 512]
                }
            }

            pub fn get_entry(&mut self, index: usize) -> Option<&mut PageFlags> {
                if self.entries.len() < index {
                    return None;
                }

                Some(&mut self.entries[index])
            }

            pub fn remove_entry(&mut self, index: usize) {
                if self.entries.len() < index {
                    return;
                }

                self.entries[index].disable_all();
            }

            pub fn recurse_next_entry(&mut self, index: usize, address: VirtualAddress) -> Result<(), QuantumError> {
                if index >= 512 {
                    return Err(QuantumError::OutOfRange);
                }

                self.entries[index].paste_address(address)
            }
        }
    )*)
}

map_impl! { PageMapLevel4 PageDirPointerTable PageDir PageTable}

#[cfg(test)]
pub mod test_case {
    use crate::debug_println;
    use crate::memory::paging::page_structures::{PageDirPointerTable, PageMapLevel4};
    use crate::memory::VirtualAddress;

    #[test_case]
    pub fn test_page_map_addressing() {
        let mut plm4 = PageMapLevel4::new();
        let plm3 = PageDirPointerTable::new();

        let ptr = plm3.entries.as_ptr();
        let address = VirtualAddress::from_ptr(ptr);

        plm4.recurse_next_entry(2, address).unwrap();

    }

}