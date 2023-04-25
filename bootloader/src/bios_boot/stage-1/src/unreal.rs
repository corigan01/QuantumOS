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
#[cfg(debug)]
use crate::bios_println;

use core::arch::asm;
use core::mem::size_of;
use quantum_lib::x86_64::interrupts::Interrupts;
use quantum_lib::x86_64::PrivlLevel;
use quantum_lib::x86_64::registers::{CpuStack, CR0, Segment, SegmentRegs};

#[repr(C, packed(2))]
pub struct SimpleGDTPtr {
    limit: u16,
    base: *const u8,
}

#[repr(C)]
pub struct SimpleGDT {
    zero: u64,
    code: u64,
    data: u64,
}

impl SimpleGDT {
    pub const fn new() -> Self {
        let limit = {
            let limit_low = 0xffff;
            let limit_high = 0xf << 48;
            limit_high | limit_low
        };
        let access_common = {
            let present = 1 << 47;
            let user_segment = 1 << 44;
            let read_write = 1 << 41;
            present | user_segment | read_write
        };
        let protected_mode = 1 << 54;
        let granularity = 1 << 55;
        let base_flags = protected_mode | granularity | access_common | limit;
        let executable = 1 << 43;
        Self {
            zero: 0,
            code: base_flags | executable,
            data: base_flags,
        }
    }

    pub unsafe fn package(&'static self) -> SimpleGDTPtr {
        SimpleGDTPtr {
            base: self as *const Self as *const u8,
            limit: (3 * size_of::<u64>() - 1) as u16,
        }
    }
}

impl SimpleGDTPtr {
    pub unsafe fn load(self) {
        asm!(
            "lgdt [{ptr}]",
            ptr = in(reg) &self, options(readonly, nostack)
        );
    }
}

#[link_section = ".GDT"]
static GD_TABLE: SimpleGDT = SimpleGDT::new();

#[no_mangle]
#[inline(never)]
pub unsafe fn enter_unreal_mode() {
    Interrupts::disable();

    let seg = SegmentRegs::save();

    GD_TABLE.package().load();

    CR0::set_protected_mode(true);

    SegmentRegs::reload_all_to(Segment::new(2, PrivlLevel::Ring0));

    CR0::set_protected_mode(false);

    SegmentRegs::restore(seg);

    Interrupts::enable();
}

#[no_mangle]
#[inline(never)]
pub unsafe fn enter_stage2(entry_point: *const u8, info: *const u8) {
    Interrupts::disable();
    CR0::set_protected_mode(true);

    CpuStack::align_stack();
    CpuStack::push(info as u32);
    CpuStack::push(entry_point as u32);

    SegmentRegs::reload_all_to(Segment::new(2, PrivlLevel::Ring0));

    asm!("ljmp $0x8, $2f", "2:", options(att_syntax));
    asm!(
        ".code32",
        "pop {0:e}",
        "call {0:e}",
        "4:",
        "jmp 4b",
        out(reg) _,
    );

}
