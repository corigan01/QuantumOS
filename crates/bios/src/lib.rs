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

#![no_std]

use arch::registers::{eflags, Regs32};
#[cfg(target_pointer_width = "32")]
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
    fn from_ax(ax: u16) -> Self {
        match ax {
            INVALID_BIOS_CALL_AX => Self::InvalidInput,
            NOT_SUPPORTED_CALL_AX => BiosStatus::NotSupported,
            _ if eflags::is_carry_set() => BiosStatus::Failed,
            _ => BiosStatus::Success,
        }
    }

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

#[cfg(target_pointer_width = "32")]
macro_rules! bios_call {
    (priv, $type:ty, $id:ident: $value:expr) => {
        let $id: $type = $value;
    };
    (priv, $type:ty, mut $id:ident: $value:expr) => {
        let mut $id: $type = $value;
    };
    (priv, $type:ty, $id:ident: ) => {
        let $id: $type = 0;
    };
    (priv, $type:ty, mut $id:ident: ) => {
        let mut $id: $type = 0;
    };

    (int: $number:literal, $(ax: $ax:expr,)? $(bx: $bx:expr,)? $(cx: $cx:expr,)? $(dx: $dx:expr,)? $(es: $es:expr,)? $(di: $di:expr,)? $(si: $si:expr)?) => {{
        bios_call!(priv, u16, mut ax: $($ax)?);
        bios_call!(priv, u16, bx: $($bx)?);
        bios_call!(priv, u16, cx: $($cx)?);
        bios_call!(priv, u16, dx: $($dx)?);
        bios_call!(priv, u16, es: $($es)?);
        bios_call!(priv, u16, di: $($di)?);
        bios_call!(priv, u16, si: $($si)?);

        #[allow(unused_assignments)]
        unsafe { ::core::arch::asm!(
            concat!("
                push es
                push bx
                push si
                mov es, {es:x}
                mov bx, {bx:x}
                mov si, {si:x}
                int 0x",$number,"
                pop si
                pop bx
                pop es
            "),
            inout("ax") ax => ax,
            in("cx") cx,
            in("dx") dx,
            in("di") di,
            bx = in(reg) bx,
            es = in(reg) es,
            si = in(reg) si,
        ) }

        ax
    }};
}

#[cfg(not(target_pointer_width = "32"))]
macro_rules! bios_call {
    (priv, $type:ty, $id:ident: $value:expr) => {
        #[allow(unused)]
        let $id: $type = $value;
    };
    (priv, $type:ty, mut $id:ident: $value:expr) => {
        #[allow(unused)]
        let mut $id: $type = $value;
    };
    (priv, $type:ty, $id:ident: ) => {
        #[allow(unused)]
        let $id: $type = 0;
    };
    (priv, $type:ty, mut $id:ident: ) => {
        #[allow(unused)]
        let mut $id: $type = 0;
    };

    (int: $number:literal, $(ax: $ax:expr,)? $(bx: $bx:expr,)? $(cx: $cx:expr,)? $(dx: $dx:expr,)? $(es: $es:expr,)? $(di: $di:expr,)? $(si: $si:expr)?) => {{
        bios_call!(priv, u16, mut ax: $($ax)?);
        bios_call!(priv, u16, bx: $($bx)?);
        bios_call!(priv, u16, cx: $($cx)?);
        bios_call!(priv, u16, dx: $($dx)?);
        bios_call!(priv, u16, es: $($es)?);
        bios_call!(priv, u16, di: $($di)?);
        bios_call!(priv, u16, si: $($si)?);

        fn unsupported() {
            panic!("Unsupported on current target, please use 16-bit!");
        }

        unsupported();

        0
    }};
}

// FIXME: We should remove this function and add int0x15 into macro!
//        However, currently the macro causes the bootloader to fail.
#[inline]
pub unsafe fn int_0x15(reg: &mut Regs32, es: u16) -> BiosStatus {
    #[cfg(target_pointer_width = "32")]
    asm!(
        "push es",
        "mov es, {es:e}",
        "int 0x15",
        "pop es",
        inout("eax") reg.eax => reg.eax,
        inout("ebx") reg.ebx => reg.ebx,
        inout("ecx") reg.ecx => reg.ecx,
        inout("edx") reg.edx => reg.edx,
        inout("edi") reg.edi => reg.edi,
        es = in(reg) es,
    );

    #[cfg(not(target_pointer_width = "32"))]
    {
        let _ = es;
        let _ = reg;
        panic!("Unsupported on current target, please use 16-bit!");
    }

    #[cfg(target_pointer_width = "32")]
    match reg.eax as u16 {
        INVALID_BIOS_CALL_AX => BiosStatus::InvalidData,
        NOT_SUPPORTED_CALL_AX => BiosStatus::NotSupported,
        _ if eflags::is_carry_set() => BiosStatus::Failed,
        _ => BiosStatus::Success,
    }
}

pub mod video {
    use core::ptr::addr_of;
    const TELETYPE_OUTPUT_CHAR: u16 = 0x0E00;

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
        bios_call! {
            int: 10,
            ax: TELETYPE_OUTPUT_CHAR | (c as u16 & 0x00FF),
        };
    }

    #[repr(C, align(256))]
    #[derive(Clone, Copy)]
    pub struct VesaMode {
        pub attributes: u16,
        pub window_a: u8,
        pub window_b: u8,
        pub granularity: u16,
        pub window_size: u16,
        pub segment_a: u16,
        pub segment_b: u16,
        pub win_function_ptr: u32,
        pub pitch: u16,
        pub width: u16,
        pub height: u16,
        pub w_char: u8,
        pub y_char: u8,
        pub planes: u8,
        pub bpp: u8,
        pub banks: u8,
        pub memory_model: u8,
        pub bank_size: u8,
        pub image_pages: u8,
        pub reserved1: u8,
        pub red_mask: u8,
        pub red_pos: u8,
        pub green_mask: u8,
        pub green_pos: u8,
        pub blue_mask: u8,
        pub blue_pos: u8,
        pub reserved_mask: u8,
        pub reserved_pos: u8,
        pub color_attributes: u8,
        pub framebuffer: u32,
        pub off_screen_memory_offset: u32,
        pub off_screen_memory_size: u16,
        reserved2: [u8; 206],
    }

    impl core::fmt::Debug for VesaMode {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("VesaMode")
                .field("x", &self.width)
                .field("y", &self.height)
                .finish()
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    pub struct Vesa {
        pub signature: [u8; 4],
        pub version: u16,
        pub oem_string_ptr: (u16, u16),
        pub capabilities: [u8; 4],
        pub video_mode_ptr: (u16, u16),
        pub size_64k_blocks: u16,
    }

    #[derive(Clone, Copy, Debug)]
    pub enum VesaErrorKind {
        NotSupported,
        Failed,
        InvalidHardware,
        Invalid,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct VesaModeId(u16);

    impl VesaModeId {
        pub unsafe fn force_mode(mode: u16) -> Self {
            Self(mode)
        }

        pub fn get_id(self) -> u16 {
            self.0
        }

        pub fn querry(self) -> Result<VesaMode, VesaErrorKind> {
            let uninit_mode: VesaMode = unsafe { core::mem::zeroed() };

            bios_call!(
                int: 10,
                ax: 0x4F01,
                cx: self.0,
                es: (addr_of!(uninit_mode) as u32 / 0x10) as u16,
                di: (addr_of!(uninit_mode) as u32 % 0x10) as u16,
            );

            if uninit_mode.attributes & 0x90 != 0x90 {
                // We don't currently want to deal with non-linear framebuffer support
                return Err(VesaErrorKind::NotSupported);
            }

            if uninit_mode.memory_model != 4 && uninit_mode.memory_model != 6 {
                // same with non-packed pixel modes
                return Err(VesaErrorKind::NotSupported);
            }

            Ok(uninit_mode)
        }

        pub fn set(self) -> Result<(), VesaErrorKind> {
            bios_call!(
                int: 10,
                ax: 0x4F02,
                bx: self.0,
            );

            Ok(())
        }
    }

    impl Vesa {
        pub fn quarry() -> Result<Self, VesaErrorKind> {
            let uninit_self: Self = Default::default();

            bios_call!(
                int: 10,
                ax: 0x4F00,
                es: (addr_of!(uninit_self) as u32 / 0x10) as u16,
                di: (addr_of!(uninit_self) as u32 % 0x10) as u16,
            );

            if &uninit_self.signature == b"VESA" && uninit_self.version == 0x0300 {
                Ok(uninit_self)
            } else {
                Err(VesaErrorKind::Invalid)
            }
        }

        pub fn modes(&self) -> impl Iterator<Item = VesaModeId> {
            let modes_loc = (self.video_mode_ptr.1 as u32 * 0x10) + self.video_mode_ptr.0 as u32;
            let modes_ptr = modes_loc as *const VesaModeId;

            let mut mode_len = 0;
            loop {
                unsafe {
                    if (*modes_ptr.add(mode_len)).0 == 0xFFFF {
                        break;
                    }

                    mode_len += 1;
                }
            }

            unsafe { core::slice::from_raw_parts(modes_ptr, mode_len) }
                .into_iter()
                .copied()
        }
    }
}

pub mod disk {
    use crate::BiosStatus;
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

    pub unsafe fn raw_read(disk_id: u16, lba: u64, count: usize, ptr: *mut u8) -> BiosStatus {
        let package = DiskAccessPacket::new(count as u16, lba, ptr as u32);

        assert!(addr_of!(package) as u32 & 0xFFFF == addr_of!(package) as u32);

        BiosStatus::from_ax(bios_call! {
            int: 13,
            ax: DISK_DAP_READ,
            dx: disk_id,
            si: addr_of!(package) as u16
        })
    }
}

pub mod memory {
    use crate::BiosStatus;

    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct MemoryEntry {
        pub base_address: u64,
        pub region_length: u64,
        pub region_type: u32,
        pub acpi_attributes: u32,
    }

    impl mem::phys::MemoryDesc for &MemoryEntry {
        fn memory_kind(&self) -> mem::phys::PhysMemoryKind {
            match self.region_type {
                0 => mem::phys::PhysMemoryKind::None,
                1 => mem::phys::PhysMemoryKind::Free,
                2 => mem::phys::PhysMemoryKind::Reserved,
                3 => mem::phys::PhysMemoryKind::AcpiReclaimable,
                _ => mem::phys::PhysMemoryKind::Broken,
            }
        }

        fn memory_start(&self) -> u64 {
            self.base_address
        }

        fn memory_end(&self) -> u64 {
            self.base_address + self.region_length
        }
    }

    impl MemoryEntry {
        pub const REGION_RESERVED: u32 = 0x2;
        pub const REGION_FREE: u32 = 0x1;
    }

    // FIXME: We should not be returning a Result with BiosStatus as the error, but instead
    //        it should be a type containing the error kind.
    unsafe fn read_region(ptr: *mut MemoryEntry, ebx: u32) -> Result<u32, BiosStatus> {
        use crate::int_0x15;
        use arch::registers::Regs32;

        let low_ptr = (ptr as u32) % 0x10;
        let high_ptr = ((ptr as u32) / 0x10) as u16;

        let mut regs = Regs32 {
            eax: 0xE820,
            ebx,
            ecx: 24,
            edx: 0x534D4150,
            edi: low_ptr,
            ..Regs32::default()
        };

        match int_0x15(&mut regs, high_ptr) {
            BiosStatus::Success => Ok(regs.ebx),
            err => Err(err),
        }
    }

    /// # Read Mapping
    /// Reads the computer's memory map using Bios-Call-0x15's 0xE820 command.
    ///
    /// Returns the amount of memory entries read.
    ///
    /// # Safety
    /// This function will only read memory regions it has room to fit in the
    /// provided buffer. If there are more regions than will fit in the buffer
    /// this function will simply return and return the size of the buffer.
    pub fn read_mapping(memory: &mut [MemoryEntry]) -> Result<usize, BiosStatus> {
        let mut ebx = 0;

        for (en, entry) in memory.iter_mut().enumerate() {
            let entry_ptr = entry as *mut MemoryEntry;

            ebx = unsafe { read_region(entry_ptr, ebx) }?;

            if ebx == 0 {
                return Ok(en + 1);
            }
        }

        Ok(memory.len())
    }
}
