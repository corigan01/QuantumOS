/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
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

use core::arch::asm;

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default)]
#[repr(C)]
pub struct GPRegs8 {
    pub al: u8,
    pub ah: u8,
    pub bl: u8,
    pub bh: u8,
    pub cl: u8,
    pub ch: u8,
    pub dl: u8,
    pub dh: u8,
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default)]
#[repr(C)]
pub struct GPRegs16 {
    pub ax: u16,
    pub bx: u16,
    pub cx: u16,
    pub dx: u16,
    pub bp: u16,
    pub sp: u16,
    pub si: u16,
    pub di: u16,
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default)]
#[repr(C)]
pub struct GPRegs32 {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebp: u32,
    pub esp: u32,
    pub esi: u32,
    pub edi: u32,
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default)]
#[repr(C)]
pub struct GPRegs64 {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default)]
#[repr(C)]
pub struct SegmentRegs {
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub ss: u16,
    pub fs: u16,
    pub gs: u16,
}

pub struct CR0 {}

impl CR0 {
    #[inline(never)]
    pub fn read_to_32() -> u32 {
        let reading_value;

        unsafe {
            asm!(
                "mov {output:e}, cr0",
                output = out(reg) reading_value
            );
        }

        assert!(reading_value != 0);

        reading_value
    }

    #[inline(never)]
    fn write_at_position(bit_pos: u32, flag: bool) {
        assert!(bit_pos <= 32);

        let read_value = Self::read_to_32();
        let set_bit_pos = 1 << bit_pos;

        let moved_bit = if flag {
            set_bit_pos
        } else {
            u32::MAX ^ set_bit_pos
        };

        let new_value = if flag {
            read_value | moved_bit
        } else {
            read_value & moved_bit
        };

        unsafe {
            asm!(
                "mov cr0, {value:e}",
                value = in(reg) new_value
            );
        }
    }

    pub fn is_protected_mode_set() -> bool {
        let value = Self::read_to_32();

        value & 1 == 1
    }

    pub fn is_monitor_coprocessor_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 1;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    pub fn is_enumlation_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 2;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    pub fn is_task_switched_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 3;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    pub fn is_extention_type_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 4;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    pub fn is_not_write_through_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 16;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    pub fn is_alignment_mask_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 18;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    pub fn is_write_protect_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 28;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    pub fn set_protected_mode(flag: bool) {
        Self::write_at_position(0, flag);
    }

    pub fn set_monitor_coprocessor(flag: bool) {
        Self::write_at_position(1, flag);
    }

    pub fn set_enumlation(flag: bool) {
        Self::write_at_position(2, flag);
    }

    pub fn set_task_switched(flag: bool) {
        Self::write_at_position(3, flag);
    }

    pub fn set_extention_type(flag: bool) {
        Self::write_at_position(4, flag);
    }

    pub fn set_not_write_through(flag: bool) {
        Self::write_at_position(16, flag);
    }

    pub fn set_alignment_mask(flag: bool) {
        Self::write_at_position(18, flag);
    }

    pub fn set_write_protect(flag: bool) {
        Self::write_at_position(28, flag);
    }
}
