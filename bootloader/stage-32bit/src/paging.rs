/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
    Part of the Quantum OS Project

Copyright 2024 Gavin Kellam

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

use arch::{
    paging64::{PageEntry2M, PageEntryLvl3, PageEntryLvl4, PageMapLvl2, PageMapLvl3, PageMapLvl4},
    registers::{cr0, cr3, cr4, ia32_efer, Segment, SegmentRegisters},
    CpuPrivilege,
};
use core::cell::SyncUnsafeCell;
use lignan::{log, logln};
use util::consts::{GIB, MIB};

/// Amount of Gib to identity map
const IDMAP_GIG_AMOUNT: usize = 1;

static TABLE_LVL4: SyncUnsafeCell<PageMapLvl4> = SyncUnsafeCell::new(PageMapLvl4::new());
static TABLE_LVL3: SyncUnsafeCell<PageMapLvl3> = SyncUnsafeCell::new(PageMapLvl3::new());
static TABLE_LVL2: SyncUnsafeCell<[PageMapLvl2; IDMAP_GIG_AMOUNT]> =
    SyncUnsafeCell::new([PageMapLvl2::new(); IDMAP_GIG_AMOUNT]);

pub fn identity_map() {
    for gig in 0..IDMAP_GIG_AMOUNT {
        let table_ptr = unsafe { &raw mut (*TABLE_LVL2.get())[gig] };

        for mb2 in 0..512 {
            let phy_addr = (mb2 as u64 * 2 * (MIB as u64)) + (gig as u64 * (GIB as u64));

            let lvl2_entry = PageEntry2M::new()
                .set_present_flag(true)
                .set_read_write_flag(true)
                .set_phy_address(phy_addr);

            unsafe { (*table_ptr).store(lvl2_entry, mb2) };
        }

        let lvl3_entry = PageEntryLvl3::new()
            .set_present_flag(true)
            .set_read_write_flag(true)
            .set_next_entry_phy_address(unsafe { (*table_ptr).table_ptr() });

        unsafe { (*TABLE_LVL3.get()).store(lvl3_entry, gig) };
    }

    let lvl4_entry = PageEntryLvl4::new()
        .set_present_flag(true)
        .set_read_write_flag(true)
        .set_next_entry_phy_address(unsafe { (*TABLE_LVL3.get()).table_ptr() });

    unsafe { (*TABLE_LVL4.get()).store(lvl4_entry, 0) };
}

pub unsafe fn set_page_base_reg() {
    let phy_addr = unsafe { (*TABLE_LVL4.get()).table_ptr() };

    cr3::set_page_directory_base_register(phy_addr);
}

pub unsafe fn enable_paging() {
    log!("Identity Mapping Regions...");
    identity_map();
    logln!("OK");

    log!("Setting Paging Base Register...");
    set_page_base_reg();
    logln!("OK");

    log!("Disabling Paging...");
    cr0::set_paging_flag(false);
    logln!("OK");

    log!("Setting PAE...");
    cr4::set_physical_address_extension_flag(true);
    logln!("OK");

    log!("Setting Long Mode...");
    ia32_efer::set_long_mode_enable_flag(true);
    logln!("OK");

    log!("Enabling Protected Mode...");
    cr0::set_protected_mode_flag(true);
    logln!("OK");

    log!("Enabling Paging...");
    cr0::set_paging_flag(true);
    logln!("OK");

    log!("Reloading Segments...");
    SegmentRegisters::set_data_segments(Segment::new(2, CpuPrivilege::Ring0));
    logln!("OK");
}
