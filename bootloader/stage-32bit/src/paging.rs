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
    paging64::{PageEntry1G, PageMapLvl4},
    registers::cr3,
};
use core::cell::SyncUnsafeCell;
use util::consts::GIB;

/// Amount of Gib to identity map
const IDMAP_GIG_AMOUNT: usize = 8;

static TABLE_LVL4: SyncUnsafeCell<PageMapLvl4> = SyncUnsafeCell::new(PageMapLvl4::new());

pub fn identity_map() {
    for gig in 0..IDMAP_GIG_AMOUNT {
        let v_addr = gig * GIB;

        let lvl3 = PageEntry1G::new()
            .set_present_flag(true)
            .set_read_write_flag(true)
            .set_execute_disable_flag(true)
            .set_user_accessed_flag(false)
            .set_virt_address(v_addr as u32);

        unsafe {
            (*TABLE_LVL4.get()).store(lvl3, gig);
        }
    }
}

pub unsafe fn set_page_base_reg() {
    let phy_addr = TABLE_LVL4.get() as u64;

    cr3::set_page_directory_base_register(phy_addr);
}
