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

/// CPUs context
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ProcessContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,

    // Trap frame
    pub exception_code: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflag: u64,
    pub rsp: u64,
    pub ss: u64,
}

impl ProcessContext {
    /// Make a new empty process context
    pub const fn new() -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rbp: 0,
            rdi: 0,
            rsi: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,
            cs: 0,
            ss: 0,
            rflag: 0,
            rip: 0,
            exception_code: 0,
            rsp: 0,
        }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Segment(pub u16);

impl Segment {
    pub fn new(gdt_index: u16, privl: CpuPrivilege) -> Self {
        let privl: u16 = privl.into();

        Self((gdt_index << 3) | privl)
    }

    pub const fn gdt_index(&self) -> u16 {
        self.0 >> 3
    }

    pub const fn cpu_privl(&self) -> CpuPrivilege {
        match self.0 & 3 {
            0 => CpuPrivilege::Ring0,
            1 => CpuPrivilege::Ring1,
            2 => CpuPrivilege::Ring2,
            3 => CpuPrivilege::Ring3,
            _ => unreachable!(),
        }
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

pub mod cr2 {
    #[inline(always)]
    pub fn read() -> u64 {
        #[cfg(target_pointer_width = "32")]
        let mut flags: u32;
        #[cfg(target_pointer_width = "64")]
        let mut flags: u64;

        #[cfg(target_pointer_width = "32")]
        unsafe {
            core::arch::asm!("
                mov eax, cr2
            ",
                out("eax") flags
            )
        }

        #[cfg(target_pointer_width = "64")]
        unsafe {
            core::arch::asm!("
                mov rax, cr2
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
    field(RW, 3, pub page_level_write_through),
    field(RW, 4, pub page_level_cache_disable),
    field(RWNS, 12..=63, pub page_directory_base_register)
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
        #[cfg(not(target_pointer_width = "64"))]
        let mut flags: u32;
        #[cfg(target_pointer_width = "64")]
        let mut flags: u64;

        #[cfg(not(target_pointer_width = "64"))]
        unsafe {
            core::arch::asm!("
                pushf
                pop {0:x}
            ",
                out(reg) flags
            )
        }

        #[cfg(target_pointer_width = "64")]
        unsafe {
            core::arch::asm!("
                pushf
                pop rax
            ",
                out("rax") flags
            )
        }

        #[cfg(not(target_pointer_width = "64"))]
        return flags;
        #[cfg(target_pointer_width = "64")]
        return flags as u32;
    }
}

#[inline(always)]
pub unsafe fn read_msr(msr_number: u32) -> u64 {
    let lo: u32;
    let hi: u32;

    unsafe {
        core::arch::asm!("rdmsr",
            in("ecx") msr_number,
            out("eax") lo,
            out("edx") hi
        )
    }

    lo as u64 | ((hi as u64) << 32)
}

#[inline(always)]
pub unsafe fn write_msr(msr_number: u32, value: u64) {
    let lo = value as u32;
    let hi = (value >> 32) as u32;

    core::arch::asm!("wrmsr",
        in("ecx") msr_number,
        in("eax") lo,
        in("edx") hi
    )
}

#[make_hw(
    field(RW, 0, pub syscall_extensions),
    field(RW, 8, pub long_mode_enable),
    field(RW, 10, pub long_mode_active),
    field(RW, 11, pub no_execute),
    field(RW, 12, pub secure_virtual_machine),
    field(RW, 13, pub long_mode_segment_limit),
    field(RW, 14, pub fast_fxsave),
    field(RW, 15, pub translation_cache)
)]
pub mod ia32_efer {
    use super::{read_msr, write_msr};

    #[inline(always)]
    pub fn read() -> u64 {
        unsafe { read_msr(0xC0000080) }
    }

    #[inline(always)]
    pub unsafe fn write(value: u64) {
        write_msr(0xC0000080, value);
    }
}

pub mod amd_syscall {
    use crate::CpuPrivilege;

    use super::{make_hw, read_msr, write_msr, Segment};
    const STAR: u32 = 0xC0000081;
    const LSTAR: u32 = 0xC0000082;
    const CSTAR: u32 = 0xC0000083;
    const SFMASK: u32 = 0xC0000084;

    #[make_hw(
        field(RW, 32..48, pub syscall_target_code_segment),
        field(RW, 48..64, pub sysret_target_code_segment),
    )]
    mod ia32_star {
        use super::{read_msr, write_msr, STAR};

        #[inline(always)]
        pub fn read() -> u64 {
            unsafe { read_msr(STAR) }
        }

        #[inline(always)]
        pub unsafe fn write(value: u64) {
            write_msr(STAR, value);
        }
    }

    /// Set the entry point segments for the kernel when a userspace program calls 'syscall'
    pub unsafe fn write_kernel_segments(code_segment: Segment, stack_segment: Segment) {
        assert_eq!(
            code_segment.gdt_index(),
            stack_segment.gdt_index() - 1,
            "The stack segment is always 1 index higher in the gdt from the code segment!"
        );
        assert_eq!(
            code_segment.cpu_privl(),
            CpuPrivilege::Ring0,
            "Kernel code segment should be of privilege RING0"
        );
        assert_eq!(
            stack_segment.cpu_privl(),
            CpuPrivilege::Ring0,
            "Kernel stack segment should be of privilege RING0"
        );

        unsafe { ia32_star::set_syscall_target_code_segment(code_segment.0) };
    }

    /// Set the entry point segments for userspace when a kernel syscall returns with 'sysret'
    pub unsafe fn write_userspace_segments(code_segment: Segment, stack_segment: Segment) {
        assert_eq!(
            code_segment.gdt_index() - 1,
            stack_segment.gdt_index(),
            "The stack segment is always 1 index lower in the gdt from the code segment!"
        );
        assert_eq!(
            code_segment.cpu_privl(),
            CpuPrivilege::Ring3,
            "Userspace code segment should be of privilege RING3"
        );
        assert_eq!(
            stack_segment.cpu_privl(),
            CpuPrivilege::Ring3,
            "Userspace stack segment should be of privilege RING3"
        );

        unsafe {
            ia32_star::set_sysret_target_code_segment(
                Segment::new(stack_segment.gdt_index() - 1, CpuPrivilege::Ring3).0,
            )
        };
    }

    /// Set the syscall entry point for when a userspace program calls 'syscall'.
    ///
    /// # Note
    /// - 'syscall' does not save the RSP, so kernel syscall handler will need to do
    ///    that.
    /// - 'syscall' puts userspace RIP into RCX and userspace rFLAGS into R11.
    pub unsafe fn set_syscall_target_ptr(target_fn: u64) {
        unsafe { write_msr(LSTAR, target_fn) };
    }

    /// Set the syscall entry point for when a userspace program calls 'syscall' in ia32e-32bit mode.
    ///
    /// # Note
    /// - 'syscall' does not save the RSP, so kernel syscall handler will need to do
    ///    that.
    /// - 'syscall' puts userspace RIP into RCX and userspace rFLAGS into R11.
    pub unsafe fn set_syscall_compatibility_target_ptr(target_fn: u64) {
        unsafe { write_msr(CSTAR, target_fn) };
    }

    /// If a bit is set, it clears that flag in the rFLAGS register.
    pub unsafe fn set_rflags_mask(mask: u64) {
        unsafe { write_msr(SFMASK, mask) };
    }
}
