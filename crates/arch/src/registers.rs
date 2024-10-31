/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
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

use hw::hw_device;

use crate::CpuPrivilege;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Regs8 {
    pub al: u8,
    pub ah: u8,
    pub bl: u8,
    pub bh: u8,
    pub cl: u8,
    pub ch: u8,
    pub dl: u8,
    pub dh: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Regs16 {
    pub ax: u16,
    pub bx: u16,
    pub cx: u16,
    pub dx: u16,
    pub bp: u16,
    pub sp: u16,
    pub si: u16,
    pub di: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Regs32 {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebp: u32,
    pub esp: u32,
    pub esi: u32,
    pub edi: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Regs64 {
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SegmentRegisters {
    cs: u16,
    ds: u16,
    es: u16,
    ss: u16,
    fs: u16,
    gs: u16,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Segment(pub u16);

impl Segment {
    pub fn new(gdt_index: u16, privl: CpuPrivilege) -> Self {
        let privl: u16 = privl.into();

        Self((gdt_index << 3) | privl)
    }
}

impl SegmentRegisters {
    pub unsafe fn set_data_segments(segment: Segment) {
        core::arch::asm!(
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "mov fs, ax",
            "mov gs, ax",
            in("ax") segment.0
        )
    }
}

macro_rules! flag_get {
    ($method_name: ident, $bit: literal) => {
        #[doc = concat!("# Get ", stringify!($method_name))]
        /// Check if this flag is set, or unset by reading the state of this register.
        pub fn $method_name() -> bool {
            read() & $bit != 0
        }
    };
}

macro_rules! flag_set {
    ($method_name: ident, $bit: literal) => {
        #[doc = concat!("# Get ", stringify!($method_name))]
        /// Set this flag if 'flag' is set to true, or unset this flag if 'flag' is set to false.
        pub unsafe fn $method_name(flag: bool) {
            use bits::BitManipulation;
            write(*read().set_bit($bit, flag));
        }
    };
}

macro_rules! flag_both {
    ($method_get: ident, $method_set: ident, $bit: literal) => {
        flag_set!($method_set, $bit);
        flag_get!($method_get, $bit);
    };
}

pub mod eflags {
    #[inline(never)]
    pub fn read() -> usize {
        let mut flags;

        unsafe {
            core::arch::asm!("
                pushf
                pop {0:x}
            ",
                out(reg) flags
            )
        }

        flags
    }

    flag_get!(carry, 0);
    flag_get!(parity, 2);
    flag_get!(auxiliary, 4);
    flag_get!(zero, 6);
    flag_get!(sign, 7);
    flag_get!(trap, 8);
    flag_get!(interrupts_enabled, 9);
    flag_get!(direction, 10);
    flag_get!(overflow, 11);
    flag_get!(nested_task, 14);
    flag_get!(resume, 16);
    flag_get!(v8086_mode, 17);
    flag_get!(alignment_check, 18);
    flag_get!(virt_interrupt, 19);
    flag_get!(virt_pending, 20);
    flag_get!(cpuid_available, 21);
}

hw_device! {
    pub mod cr0 {
        #[inline(always)]
        pub fn read() -> u64 {
            #[cfg(target_pointer_width = "32")]
            let mut flags: u32;
            #[cfg(target_pointer_width = "64")]
            let mut flags: u64;

            #[cfg(target_pointer_width = "32")]
            unsafe {
                core::arch::asm!("
                    mov eax, cr0
                ",
                    out("eax") flags
                )
            }

            #[cfg(target_pointer_width = "64")]
            unsafe {
                core::arch::asm!("
                    mov rax, cr0
                ",
                    out("rax") flags
                )
            }

            #[cfg(target_pointer_width = "32")]
            return flags as u64;

            #[cfg(target_pointer_width = "64")]
            flags
        }

        #[inline(always)]
        pub unsafe fn write(value: u64) {
            #[cfg(target_pointer_width = "32")]
            core::arch::asm!(
                "mov cr0, eax",
                in("eax") (value as u32)
            );

            #[cfg(target_pointer_width = "64")]
            core::arch::asm!(
                "mov cr0, rax",
                in("rax") value
            );
        }
    }

    #[field(RW, 0, cr0)]
    pub protected_mode,

    #[field(RW, 1, cr0)]
    pub monitor_co_processor,

    #[field(RW, 2, cr0)]
    pub x87_fpu_emulation,

    #[field(RW, 3, cr0)]
    pub task_switched,

    #[field(RW, 4, cr0)]
    pub extension_type,

    #[field(RW, 5, cr0)]
    pub numeric_error,

    #[field(RW, 16, cr0)]
    pub write_protect,

    #[field(RW, 18, cr0)]
    pub alignmnet_mask,

    #[field(RW, 29, cr0)]
    pub non_write_through,

    #[field(RW, 30, cr0)]
    pub cache_disable,

    #[field(RW, 31, cr0)]
    pub paging,
}
