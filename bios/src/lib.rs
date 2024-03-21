#![no_std]

use arch::registers::{eflags, Regs16};
use core::arch::asm;

// FIXME: Should only build in 16bit x86 systems!

pub const INVALID_BIOS_CALL_AX: u16 = 0x80;
pub const NOT_SUPPORTED_CALL_AX: u16 = 0x86;

pub enum BiosErrorKind {
    InvalidInput,
    InvalidData,
    NotSupported,
    Failed,
}

type Result<T> = core::result::Result<T, BiosErrorKind>;

#[inline]
pub unsafe fn int_0x10(mut reg: Regs16) -> Result<()> {
    asm!("
        push bx
        mov bx, {0:x}
        int 0x10
        pop bx
        ",
        in(reg) reg.bx,
        inout("ax") reg.ax => reg.ax,
        in("cx") reg.cx,
        in("dx") reg.dx,
        in("di") reg.di,
    );

    match reg.ax {
        INVALID_BIOS_CALL_AX => Err(BiosErrorKind::InvalidData),
        NOT_SUPPORTED_CALL_AX => Err(BiosErrorKind::NotSupported),
        _ if eflags::carry() => Err(BiosErrorKind::Failed),
        _ => Ok(()),
    }
}

pub mod video {
    use crate::int_0x10;
    use arch::registers::Regs16;

    const TELETYPE_OUTPUT: u16 = 0x0E00;

    #[inline]
    pub fn putc(c: u8) {
        unsafe {
            core::arch::asm!("
                mov ah, 0x0e
                int 0x10
            ",
                in("al") c
            );
        }
    }

    #[inline]
    pub fn print_char(c: char) {
        let regs = Regs16 {
            ax: TELETYPE_OUTPUT | (c as u16 & 0x00FF),
            ..Regs16::default()
        };

        let _ = unsafe { int_0x10(regs) };
    }
}
