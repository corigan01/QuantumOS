/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

use core::cell::SyncUnsafeCell;

use arch::{
    paging64::{PageEntry2M, PageEntryLvl3, PageEntryLvl4, PageMapLvl2, PageMapLvl3, PageMapLvl4},
    registers::cr3,
};
use util::{
    consts::{GIB, MIB, PAGE_2M},
    is_align_to,
};

/// Amount of Gib to identity map
const IDMAP_GIG_AMOUNT: usize = 1;

// Main Table
static TABLE_LVL4: SyncUnsafeCell<PageMapLvl4> = SyncUnsafeCell::new(PageMapLvl4::new());

// Tables for lower memory id-mapping
static TABLE_LVL3_ID: SyncUnsafeCell<PageMapLvl3> = SyncUnsafeCell::new(PageMapLvl3::new());
static TABLE_LVL2_ID: SyncUnsafeCell<[PageMapLvl2; IDMAP_GIG_AMOUNT]> =
    SyncUnsafeCell::new([PageMapLvl2::new(); IDMAP_GIG_AMOUNT]);

// Tables for higher-half kernel
static TABLE_LVL3_KERN: SyncUnsafeCell<PageMapLvl3> = SyncUnsafeCell::new(PageMapLvl3::new());
static TABLE_LVL2_KERN: SyncUnsafeCell<PageMapLvl2> = SyncUnsafeCell::new(PageMapLvl2::new());

#[derive(Debug, Copy, Clone)]
pub struct PageTableConfig {
    pub kernel_exe_phys: (u64, usize),
    pub kernel_stack_phys: (u64, usize),
    pub kernel_init_phys: (u64, usize),
    pub kernel_virt: u64,
}

#[derive(Debug, Copy, Clone)]
pub struct KernelVirtInfo {
    pub exe_start_virt: u64,
    pub exe_end_virt: u64,
    pub _stack_start_virt: u64,
    pub stack_end_virt: u64,
    pub init_start_virt: u64,
    pub init_end_virt: u64,
}

impl KernelVirtInfo {
    pub fn _exe_slice(&mut self) -> &'static mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.exe_start_virt as *mut u8,
                (self.exe_end_virt - self.exe_start_virt) as usize,
            )
        }
    }

    pub fn _stack_slice(&mut self) -> &'static mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self._stack_start_virt as *mut u8,
                (self.stack_end_virt - self._stack_start_virt) as usize,
            )
        }
    }
}

pub fn build_page_tables(c: PageTableConfig) -> KernelVirtInfo {
    assert!(
        c.kernel_exe_phys.1 <= GIB,
        "TODO: Currently do not support kernel's size above 1Gib"
    );
    assert!(
        c.kernel_stack_phys.1 <= GIB,
        "TODO: Currently do not support kernel's stack size above 1Gib"
    );
    assert!(is_align_to(c.kernel_exe_phys.0, PAGE_2M));
    assert!(is_align_to(c.kernel_stack_phys.0, PAGE_2M));
    assert!(is_align_to(c.kernel_virt, PAGE_2M));

    // ID MAP
    for gig in 0..IDMAP_GIG_AMOUNT {
        let table_ptr = unsafe { &raw mut (*TABLE_LVL2_ID.get())[gig] };

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

        unsafe { (*TABLE_LVL3_ID.get()).store(lvl3_entry, gig) };
    }

    let lvl4_entry = PageEntryLvl4::new()
        .set_present_flag(true)
        .set_read_write_flag(true)
        .set_next_entry_phy_address(unsafe { (*TABLE_LVL3_ID.get()).table_ptr() });

    unsafe { (*TABLE_LVL4.get()).store(lvl4_entry, 0) };

    // KERNEL MAP (EXE)
    let tbl2_offset = PageMapLvl2::addr2index(c.kernel_virt % PageMapLvl2::SIZE_FOR_TABLE).unwrap();
    let tbl3_offset = PageMapLvl4::addr2index(c.kernel_virt % PageMapLvl3::SIZE_FOR_TABLE).unwrap();
    let tbl4_offset = PageMapLvl4::addr2index(c.kernel_virt % PageMapLvl4::SIZE_FOR_TABLE).unwrap();

    let exe_pages = ((c.kernel_exe_phys.1 - 1) / PAGE_2M) + 1;
    let stack_pages = ((c.kernel_stack_phys.1 - 1) / PAGE_2M) + 1;
    let init_pages = ((c.kernel_init_phys.1 - 1) / PAGE_2M) + 1;

    for mb2 in 0..exe_pages {
        let phy_addr = c.kernel_exe_phys.0 + (mb2 * PAGE_2M) as u64;

        let lvl2_entry = PageEntry2M::new()
            .set_present_flag(true)
            .set_read_write_flag(true)
            .set_phy_address(phy_addr);

        unsafe { (*TABLE_LVL2_KERN.get()).store(lvl2_entry, mb2 + tbl2_offset) };
    }

    // KERNEL MAP (STACK)
    for mb2 in 0..stack_pages {
        let phy_addr = c.kernel_stack_phys.0 + (mb2 * PAGE_2M) as u64;

        let lvl2_entry = PageEntry2M::new()
            .set_present_flag(true)
            .set_read_write_flag(true)
            .set_phy_address(phy_addr);

        unsafe { (*TABLE_LVL2_KERN.get()).store(lvl2_entry, mb2 + exe_pages + 1 + tbl2_offset) };
    }

    // KERNEL MAP (INIT)
    for mb2 in 0..init_pages {
        let phy_addr = c.kernel_init_phys.0 + (mb2 * PAGE_2M) as u64;

        let lvl2_entry = PageEntry2M::new()
            .set_present_flag(true)
            .set_read_write_flag(true)
            .set_phy_address(phy_addr);

        unsafe {
            (*TABLE_LVL2_KERN.get())
                .store(lvl2_entry, mb2 + exe_pages + 2 + stack_pages + tbl2_offset)
        };
    }

    let lvl3_kernel_entry = PageEntryLvl3::new()
        .set_present_flag(true)
        .set_read_write_flag(true)
        .set_next_entry_phy_address(unsafe { (*TABLE_LVL2_KERN.get()).table_ptr() });

    unsafe { (*TABLE_LVL3_KERN.get()).store(lvl3_kernel_entry, tbl3_offset) };

    let lvl4_entry = PageEntryLvl4::new()
        .set_present_flag(true)
        .set_read_write_flag(true)
        .set_next_entry_phy_address(unsafe { (*TABLE_LVL3_KERN.get()).table_ptr() });

    unsafe { (*TABLE_LVL4.get()).store(lvl4_entry, tbl4_offset) };

    KernelVirtInfo {
        exe_start_virt: c.kernel_virt,
        exe_end_virt: c.kernel_virt + (exe_pages * PAGE_2M) as u64,
        _stack_start_virt: c.kernel_virt + ((exe_pages + 1) * PAGE_2M) as u64,
        stack_end_virt: c.kernel_virt + ((exe_pages + stack_pages + 1) * PAGE_2M) as u64,
        init_start_virt: c.kernel_virt + ((exe_pages + stack_pages + 2) * PAGE_2M) as u64,
        init_end_virt: c.kernel_virt
            + ((exe_pages + stack_pages + init_pages + 2) * PAGE_2M) as u64,
    }
}

pub unsafe fn load_page_tables() {
    let phy_addr = unsafe { (*TABLE_LVL4.get()).table_ptr() };

    unsafe { cr3::set_page_directory_base_register(phy_addr) };
}
