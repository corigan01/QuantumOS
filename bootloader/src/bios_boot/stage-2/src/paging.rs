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


use quantum_lib::{debug_print, debug_println};
use quantum_lib::address_utils::virtual_address::VirtAddress;
use quantum_lib::x86_64::paging::config::PageConfigBuilder;
use quantum_lib::x86_64::paging::structures::{PageMapLevel2, PageMapLevel3, PageMapLevel4};
use quantum_lib::x86_64::registers::{CR0, CR3, CR4, IA32_EFER, SegmentRegs};

static mut LEVEL4: PageMapLevel4 = PageMapLevel4::new();
static mut LEVEL3: PageMapLevel3 = PageMapLevel3::new();
static mut LEVEL2: [PageMapLevel2; 5] = [PageMapLevel2::new(); 5];

pub unsafe fn enable_paging() {
    debug_print!("building pages ...");

    let mut level4 = &mut LEVEL4;
    let mut level3 = &mut LEVEL3;
    let mut level2_tables = &mut LEVEL2;

    for (offset, level2) in level2_tables.iter_mut().enumerate() {
        let offset_addition = offset as u64 * 1024 * 1024 * 1024;

        for i in 0..512 {
            let huge_address = VirtAddress::new(i * 2 * 1024 * 1024 + offset_addition)
                .unwrap()
                .try_aligned()
                .unwrap();

            let two_mb_entries = PageConfigBuilder::new()
                .level2()
                .present(true)
                .read_write(true)
                .executable(true)
                .user_page(false)
                .set_huge_page_address(huge_address)
                .build()
                .unwrap();

            level2.set_entry(two_mb_entries, i as usize).unwrap();
        }

        debug_print!("L2...");

        let level_2_entry = PageConfigBuilder::new()
            .level3()
            .present(true)
            .read_write(true)
            .executable(true)
            .user_page(false)
            .set_address_of_next_table(level2.get_address())
            .build()
            .unwrap();

        level3.set_entry(level_2_entry, offset).unwrap();
    }

    debug_print!("L3...");

    let level_3_config = PageConfigBuilder::new()
        .level4()
        .present(true)
        .read_write(true)
        .executable(true)
        .user_page(false)
        .set_address_of_next_table(level3.get_address())
        .build()
        .unwrap();

    level4.set_entry(level_3_config, 0).unwrap();

    debug_print!("L4...");
    let level4_address = level4.get_address().as_u64();
    debug_println!(" OK ({}Gib Mapped!)", level2_tables.len());

    debug_print!("Disabling paging ...");
    CR0::set_paging(false);
    debug_println!("OK");

    debug_print!("Setting PAE ...");
    CR4::set_physical_address_extension(true);
    CR4::set_page_size_extension(true);
    debug_println!("OK");

    debug_print!("Setting Long mode ...");
    IA32_EFER::set_long_mode_enable(true);
    debug_println!("OK");

    debug_print!("Loading CR3 ...");
    CR3::set_page_directory_base_register(level4_address as *mut u8);
    debug_println!("OK 0x{:x}", level4_address);

    debug_print!("Enabling protected mode ...");
    CR0::set_protected_mode(true);
    debug_println!("OK");

    debug_print!("Enabling paging ...");
    CR0::set_paging(true);
    debug_println!("OK");

    debug_print!("Reloading segment registers ...");
    SegmentRegs::reload_all_to(0x10);
    debug_println!("OK");
}