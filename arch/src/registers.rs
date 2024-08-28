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
            let read_value = if flag {
                read() | (1 << $bit)
            } else {
                read() & !(1 << $bit)
            };

            write(read_value);
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

pub mod cr0 {
    #[inline(never)]
    pub fn read() -> usize {
        let mut flags;

        unsafe {
            core::arch::asm!("
                mov eax, cr0
            ",
                out("eax") flags
            )
        }

        flags
    }

    #[inline(never)]
    pub unsafe fn write(value: usize) {
        core::arch::asm!(
            "mov cr0, eax",
            in("eax") value
        )
    }

    flag_both!(get_protected_mode, set_protected_mode, 0);
    flag_both!(get_monitor_co_processor, set_monitor_co_processor, 1);
    flag_both!(get_x87_fpu_emulation, set_x87_fpu_emulation, 2);
    flag_both!(get_task_switched, set_task_switched, 3);
    flag_both!(get_extension_type, set_extention_type, 4);
    flag_both!(get_numeric_error, set_numeric_error, 5);
    flag_both!(get_write_protect, set_write_protect, 16);
    flag_both!(get_alignmnet_mask, set_alignmnet_mask, 18);
    flag_both!(get_non_write_through, set_non_write_through, 29);
    flag_both!(get_cache_disable, set_cache_disable, 30);
    flag_both!(get_paging, set_paging, 31);
}
