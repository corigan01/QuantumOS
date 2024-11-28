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

use hw::make_hw;

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

#[make_hw(
    field(RW, 0, pub protected_mode),
    field(RW, 1, pub monitor_co_processor),
    field(RW, 2, pub x87_fpu_emulation),
    field(RW, 3, pub task_switch),
    field(RW, 4, pub extention_type),
    field(RW, 5, pub numeric_error),
    field(RW, 16, pub write_protect),
    field(RW, 18, pub alignment_mask),
    field(RW, 29, pub non_write_through),
    field(RW, 30, pub cache_disable),
    field(RW, 31, pub paging)
)]
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

#[make_hw(
    field(RW, 3, pub page_level_write_through),
    field(RW, 4, pub page_level_cache_disable),
    field(RW, 12..=63, pub page_directory_base_register)
)]
pub mod cr3 {
    #[inline(always)]
    pub fn read() -> u64 {
        #[cfg(target_pointer_width = "32")]
        let mut flags: u32;
        #[cfg(target_pointer_width = "64")]
        let mut flags: u64;

        #[cfg(target_pointer_width = "32")]
        unsafe {
            core::arch::asm!("
                mov eax, cr3
            ",
                out("eax") flags
            )
        }

        #[cfg(target_pointer_width = "64")]
        unsafe {
            core::arch::asm!("
                mov rax, cr3
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
            "mov cr3, eax",
            in("eax") (value as u32)
        );

        #[cfg(target_pointer_width = "64")]
        core::arch::asm!(
            "mov cr3, rax",
            in("rax") value
        );
    }
}

#[make_hw(
    field(RW, 0, pub v8086),
    field(RW, 1, pub protected_mode_virtual_interrupts),
    field(RW, 2, pub time_stamp_disable),
    field(RW, 3, pub debugging_extensions),
    field(RW, 4, pub page_size_extension),
    field(RW, 5, pub physical_address_extension),
    field(RO, 6, pub machine_check_exception),
    field(RW, 7, pub page_global_enabled),
    field(RW, 8, pub perf_mon_counter_enable),
    field(RW, 9, pub os_supporting_fxsave_fxstor),
    field(RW, 10, pub os_supporting_unmasked_simd_float),
    field(RW, 11, pub user_mode_instruction_prevention),
    field(RW, 12, pub lvl5_page_tables),
    field(RW, 13, pub virtual_machine_extentions),
    field(RW, 14, pub safer_mode_extensions),
    field(RW, 16, pub fsgsbase),
    field(RW, 17, pub pcid),
    field(RW, 18, pub xsave),
    field(RW, 20, pub supervisor_exe_protection),
    field(RW, 21, pub supervisor_access_prevention),
    field(RW, 22, pub protection_key),
    field(RW, 23, pub force_control_flow),
    field(RW, 24, pub protection_key_for_supervisor)
)]
pub mod cr4 {
    #[inline(always)]
    pub fn read() -> u64 {
        #[cfg(target_pointer_width = "32")]
        let mut flags: u32;
        #[cfg(target_pointer_width = "64")]
        let mut flags: u64;

        #[cfg(target_pointer_width = "32")]
        unsafe {
            core::arch::asm!("
                mov eax, cr4
            ",
                out("eax") flags
            )
        }

        #[cfg(target_pointer_width = "64")]
        unsafe {
            core::arch::asm!("
                mov rax, cr4
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
            "mov cr4, eax",
            in("eax") (value as u32)
        );

        #[cfg(target_pointer_width = "64")]
        core::arch::asm!(
            "mov cr4, rax",
            in("rax") value
        );
    }
}

#[make_hw(
    field(RO, 0, pub carry),
    field(RO, 2, pub parity),
    field(RO, 4, pub auxiliary),
    field(RO, 6, pub zero),
    field(RO, 7, pub sign),
    field(RO, 8, pub trap),
    field(RO, 9, pub interrupts_enable),
    field(RO, 10, pub direction),
    field(RO, 11, pub overflow),
    field(RO, 14, pub nested_task),
    field(RO, 16, pub resume),
    field(RO, 17, pub v8086_mode),
    field(RO, 18, pub alignment_check),
    field(RO, 19, pub virt_interrupt),
    field(RO, 20, pub virt_pending),
    field(RO, 21, pub cpuid_available),
)]
pub mod eflags {
    #[inline(never)]
    pub fn read() -> u32 {
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
}
