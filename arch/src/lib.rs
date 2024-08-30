#![no_std]

pub mod registers;

pub mod interrupts {
    pub unsafe fn enable_interrupts() {
        core::arch::asm!("cli");
    }

    pub unsafe fn disable_interrupts() {
        core::arch::asm!("sti");
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CpuPrivilege {
    Ring0,
    Ring1,
    Ring2,
    Ring3,
}

impl Into<u16> for CpuPrivilege {
    fn into(self) -> u16 {
        match self {
            CpuPrivilege::Ring0 => 0,
            CpuPrivilege::Ring1 => 1,
            CpuPrivilege::Ring2 => 2,
            CpuPrivilege::Ring3 => 3,
        }
    }
}
