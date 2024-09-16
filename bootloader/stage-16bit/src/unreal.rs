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

use core::{arch::asm, mem::size_of};

use arch::{
    interrupts::disable_interrupts,
    registers::{cr0, Segment, SegmentRegisters},
    stack::{align_stack, push_stack},
};
use bootloader::Stage16toStage32;

type GDEntry = u64;

#[repr(C)]
struct GlobalDT {
    entries: [GDEntry; 4],
}

impl GlobalDT {
    const fn zeroed() -> Self {
        Self { entries: [0; 4] }
    }

    const fn unreal() -> Self {
        let mut unreal = Self::zeroed();

        // FIXME: Make this easier to understand
        // segment 0x08
        unreal.entries[1] = 0xcf9a000000ffff;
        // segment 0x10
        unreal.entries[2] = 0xcf92000000ffff;

        unreal
    }

    fn package(&'static self) -> GdtPointer {
        GdtPointer {
            size: size_of::<Self>() as u16 - 1,
            ptr: self as *const GlobalDT,
        }
    }
}

#[repr(C, packed(2))]
pub struct GdtPointer {
    size: u16,
    ptr: *const GlobalDT,
}

impl GdtPointer {
    unsafe fn load(self) {
        asm!("
                cli
                lgdt [{ptr}]
            ",
            ptr = in(reg) &self
        );
    }
}

#[link_section = ".GDT"]
static GLOBAL_DESCRIPTOR_TABLE: GlobalDT = GlobalDT::unreal();

pub unsafe fn enter_unreal() {
    GLOBAL_DESCRIPTOR_TABLE.package().load();

    // Set protected mode
    let mut cr0: u32;
    asm!("mov {0:e}, cr0", out(reg) cr0);
    cr0 |= 1;
    asm!("mov cr0, {0:e}", in(reg) cr0);

    // set protected segments
    asm!("
            mov ds, {0:x}
            mov ss, {0:x}
        ",
        in(reg) 0x10
    );

    // unset protected mode
    cr0 &= !1;
    asm!("mov cr0, {0:e}", in(reg) cr0);

    // restore default segments
    asm!("
            mov ds, {0:x}
            mov ss, {0:x}
            sti
        ",
        in(reg) 0x0
    );
}

#[inline(never)]
pub unsafe fn enter_stage2(entry_point: *const u8, stage_to_stage: *const Stage16toStage32) -> ! {
    disable_interrupts();
    cr0::set_protected_mode(true);

    SegmentRegisters::set_data_segments(Segment::new(2, arch::CpuPrivilege::Ring0));

    align_stack();
    push_stack(stage_to_stage as usize);
    push_stack(entry_point as usize);

    asm!("ljmp $0x8, $2f", "2:", options(att_syntax));
    asm!("
            .code32
            pop {0:e}
            call {0:e}
        ",
        out(reg) _
    );

    panic!("Stage32 should never return");
}
