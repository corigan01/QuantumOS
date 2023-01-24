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

use crate::{debug_print, debug_println, to_giga_bytes, to_kilo_bytes, to_mega_bytes};
use crate::error_utils::QuantumError;
use crate::memory::paging::page_flags::{TableFlagOptions, TableFlags};
use crate::memory::VirtualAddress;
use owo_colors::OwoColorize;

#[repr(align(4096))]
struct PageMapLevel4 {
    entries: [TableFlags; 512]
}

#[repr(align(4096))]
struct PageDirPointerTable {
    entries: [TableFlags; 512]
}

#[repr(align(4096))]
struct PageDir {
    entries: [TableFlags; 512]
}

#[repr(align(4096))]
struct PageTable {
    entries: [TableFlags; 512]
}

type PLM4 = PageMapLevel4;
type PLM3 = PageDirPointerTable;
type PLM2 = PageDir;
type PLM1 = PageTable;

macro_rules! map_impl {
    ($($t:ty)*) => ($(
        impl $t {
             pub fn new() -> Self {
                Self {
                    entries: [TableFlags::new(); 512]
                }
            }

            pub fn get_entry(&mut self, index: usize) -> Option<&mut TableFlags> {
                if self.entries.len() <= index {
                    return None;
                }

                Some(&mut self.entries[index])
            }

            pub fn remove_entry(&mut self, index: usize) {
                if self.entries.len() < index {
                    return;
                }

                self.entries[index].reset();
            }

            pub fn recurse_next_entry(&mut self, index: usize, address: VirtualAddress) -> Result<(), QuantumError> {
                if index >= self.entries.len() {
                    return Err(QuantumError::OutOfRange);
                }

                self.entries[index].paste_address(address)
            }

            pub fn get_address(&self) -> VirtualAddress {
                 VirtualAddress::from_ptr(self.entries.as_ptr())
            }

            pub fn print_structure(&self, entry: Option<&mut TableFlags>) {
                debug_print!("{} -- {}{:016X} {}{:016X} ",
                    Self::TABLE_NAME.blue().bold().underline(),
                    "A:".white(),
                    (self.entries.as_ptr() as u64).white(),
                    "S:".white(),
                    Self::ADDRESS_SPACE.white());

                if let Some(entry) = entry {
                    debug_print!(" -- Table Flags: {:?}", entry.bold().underline());
                }

                for i in 0..self.entries.len() {
                    let entry = self.entries[i];

                    if i % 16 == 0 {
                        debug_println!("");
                    }

                    if entry.is_set(TableFlagOptions::Present) {
                        debug_print!("{}{:016X} ", "0x".bright_green(), entry.as_u64().bright_green());
                    }
                    else if entry.as_u64() > 0 {
                        debug_print!("{}{:016X} ", "0x".yellow(), entry.as_u64().yellow());
                    }
                    else {
                      debug_print!("{}{:016X} ", "0x".white().dimmed(), entry.as_u64().white().dimmed());
                    }
                }

                 debug_println!("\n");
            }

        }
    )*)
}

map_impl! { PageMapLevel4 PageDirPointerTable PageDir PageTable }

trait PageMapAddressOptions {
    type NextTableType;
    const TABLE_NAME: &'static str;
    const ADDRESS_SPACE: u64;
}

impl PageMapAddressOptions for PageMapLevel4 {
    type NextTableType = PageDirPointerTable;
    const TABLE_NAME: &'static str = "(PLM4)";
    const ADDRESS_SPACE: u64 = to_giga_bytes!(512);
}

impl PageMapAddressOptions for PageDirPointerTable {
    type NextTableType = PageDir;
    const TABLE_NAME: &'static str = "(PLM3)";
    const ADDRESS_SPACE: u64 = to_giga_bytes!(1);
}

impl PageMapAddressOptions for PageDir {
    type NextTableType = PageTable;
    const TABLE_NAME: &'static str = "(PLM2)";
    const ADDRESS_SPACE: u64 = to_mega_bytes!(2);
}

impl PageMapAddressOptions for PageTable {
    type NextTableType = ();
    const TABLE_NAME: &'static str = "(PLM1)";
    const ADDRESS_SPACE: u64 = to_kilo_bytes!(4);
}


#[cfg(test)]
pub mod test_case {
    use crate::debug_println;
    use crate::memory::init_alloc::INIT_ALLOC;
    use crate::memory::paging::page_flags::TableFlagOptions;
    use crate::memory::paging::page_structures::{PageDir, PageDirPointerTable, PageMapLevel4, PageTable, PLM4};
    use crate::memory::VirtualAddress;

    #[test_case]
    pub fn test_new_structures() {

    }

    #[test_case]
    pub fn test_page_map_addressing() {
        let mut plm4 = PageMapLevel4::new();
        let plm3 = PageDirPointerTable::new();

        plm4.recurse_next_entry(2, plm3.get_address()).unwrap();
    }

    #[test_case]
    pub fn test_page_recursion() {
        let mut plm4 = PageMapLevel4::new();
        let mut plm3 = PageDirPointerTable::new();
        let mut plm2 = PageDir::new();
        let mut plm1 = PageTable::new();

        plm4.get_entry(2).unwrap().enable(TableFlagOptions::Present);
        plm4.get_entry(4).unwrap().enable(TableFlagOptions::NoExecute);

        plm3.get_entry(2).unwrap().enable(TableFlagOptions::Present);
        plm2.get_entry(2).unwrap().enable(TableFlagOptions::Present);

        plm2.recurse_next_entry(2, plm1.get_address()).unwrap();
        plm3.recurse_next_entry(2, plm2.get_address()).unwrap();
        plm4.recurse_next_entry(2, plm3.get_address()).unwrap();

        let plm3_address = plm4.get_entry(2)
            .unwrap()
            .get_address()
            .unwrap();

        let mut recovered_plm3= unsafe {
            &mut *(plm3_address.as_ptr::<PageDirPointerTable>() as *mut PageDirPointerTable)
        };

        assert_ne!(
            recovered_plm3.get_entry(2).unwrap().as_u64(), 0_u64
        );

        debug_println!("");
        plm4.print_structure(None);
        plm3.print_structure(plm4.get_entry(2));
        plm2.print_structure(plm3.get_entry(2));
        plm1.print_structure(plm2.get_entry(2));
    }

    #[test_case]
    pub fn test_allocating_pages_live() {
        INIT_ALLOC.lock().alloc_aligned(4096);
    }

}