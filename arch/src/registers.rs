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
}
