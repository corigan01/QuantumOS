#![no_std]

use arch::registers::{eflags, Regs16, Regs32};
use core::arch::asm;

// FIXME: Should only build in 16bit x86 systems!

pub const INVALID_BIOS_CALL_AX: u16 = 0x80;
pub const NOT_SUPPORTED_CALL_AX: u16 = 0x86;

#[must_use = "BiosStatus must be used"]
#[derive(Debug)]
pub enum BiosStatus {
    Success,
    InvalidInput,
    InvalidData,
    NotSupported,
    Failed,
}

impl BiosStatus {
    pub fn unwrap(self) {
        match self {
            Self::Success => (),
            _ => panic!("Failed to unwrap BiosStatus: {:?}", self),
        }
    }

    pub fn fail(self) {
        match self {
            Self::Success => (),
            _ => {
                video::putc(b'b');
                loop {}
            }
        }
    }
}

#[inline]
pub unsafe fn int_0x10(mut reg: Regs16) -> BiosStatus {
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
        INVALID_BIOS_CALL_AX => BiosStatus::InvalidData,
        NOT_SUPPORTED_CALL_AX => BiosStatus::NotSupported,
        _ if eflags::carry() => BiosStatus::Failed,
        _ => BiosStatus::Success,
    }
}

#[inline]
pub unsafe fn int_0x13(mut reg: Regs16) -> BiosStatus {
    let status: u16;

    asm!("
        push si
        mov si, {si:x}
        int 0x13
        jc 1f
        mov {status:x}, 0
        jmp 2f
        1:
        mov {status:x}, 1
        2:
        pop si
        ",
        si = in(reg) reg.si,
        status = out(reg) status,
        inout("ax") reg.ax => reg.ax,
        in("dx") reg.dx,
    );

    match reg.ax {
        INVALID_BIOS_CALL_AX => BiosStatus::InvalidData,
        NOT_SUPPORTED_CALL_AX => BiosStatus::NotSupported,
        _ if status == 1 => BiosStatus::Failed,
        _ => BiosStatus::Success,
    }
}

#[inline]
pub unsafe fn int_0x15(reg: &mut Regs32, es: u16) -> BiosStatus {
    asm!("
        mov es, {es:x}
        push ebx
        mov ebx, {ebx:x}
        int 0x10
        mov {ebx:x}, ebx
        pop ebx
        ",
        es = in(reg) es,
        ebx = inout(reg) reg.ebx,
        inout("eax") reg.eax => reg.eax,
        in("ecx") reg.ecx,
        in("edx") reg.edx,
        in("edi") reg.edi,
    );

    match reg.eax as u16 {
        INVALID_BIOS_CALL_AX => BiosStatus::InvalidData,
        NOT_SUPPORTED_CALL_AX => BiosStatus::NotSupported,
        _ if eflags::carry() => BiosStatus::Failed,
        _ => BiosStatus::Success,
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

        unsafe { int_0x10(regs) }.unwrap();
    }
}

pub mod disk {
    use crate::{int_0x13, BiosStatus};
    use arch::registers::Regs16;
    use core::ptr::addr_of;

    const DISK_DAP_READ: u16 = 0x4200;

    #[repr(C)]
    struct DiskAccessPacket {
        packet_size: u8,
        always_zero: u8,
        sectors: u16,
        base_ptr: u16,
        base_segment: u16,
        lba: u64,
    }

    impl DiskAccessPacket {
        fn new(sectors: u16, lba: u64, ptr: u32) -> Self {
            let base_segment = (ptr >> 4) as u16;
            let base_ptr = ptr as u16 & 0xF;

            Self {
                packet_size: 0x10,
                always_zero: 0,
                sectors,
                base_ptr,
                base_segment,
                lba,
            }
        }
    }

    pub fn raw_read(disk_id: u16, lba: u64, count: usize, ptr: *mut u8) -> BiosStatus {
        let package = DiskAccessPacket::new(count as u16, lba, ptr as u32);

        assert!(
            addr_of!(package) as u32 & 0xFFFF == addr_of!(package) as u32,
            "Package address cannot be larger then 16 bit!"
        );

        let reg = Regs16 {
            dx: disk_id,
            ax: DISK_DAP_READ,
            si: (&package) as *const DiskAccessPacket as u16,
            ..Regs16::default()
        };

        unsafe { int_0x13(reg) }
    }
}

pub mod memory {
    use crate::{int_0x15, BiosStatus};
    use arch::registers::Regs32;

    #[repr(C)]
    struct MemoryEntry {
        base_address: u64,
        region_length: u64,
        region_type: u32,
        acpi_attributes: u32,
    }

    // FIXME: We should not be returning a Result with BiosStatus as the error, but instead
    //        it should be a type containing the error kind.
    unsafe fn read_region(ptr: *mut MemoryEntry, ebx: u32) -> Result<u32, BiosStatus> {
        let low_ptr = ptr as u32 % 0x10;
        let high_ptr = ptr as u16 / 0x10;

        // First Call to e820
        let mut regs = if ebx == 0 {
            Regs32 {
                eax: 0xE820,
                ebx: 0,
                edx: 0x534D4150,
                ecx: 24,
                edi: low_ptr,
                ..Regs32::default()
            }
        } else {
            Regs32 {
                eax: 0xE820,
                ebx,
                ecx: 24,
                edi: low_ptr,
                ..Regs32::default()
            }
        };

        match int_0x15(&mut regs, high_ptr) {
            BiosStatus::Success => Ok(regs.ebx),
            err => Err(err),
        }
    }
}
