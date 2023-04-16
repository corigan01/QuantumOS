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

use crate::x86_64::bios_call::BiosCallResult::{PreChecksFailed, Success};
use core::arch::asm;
use core::marker::PhantomData;
use crate::x86_64::registers::{EFLAGS, GPRegs16, GPRegs32, SegmentRegs};
use crate::Nothing;

pub struct Bit16;
pub struct Bit32;
pub struct NoCall;

#[doc = "The output from a bios call"]
#[must_use = "You must handle failed bios calls"]
#[derive(Clone, Copy, Debug)]
pub enum BiosCallResult<T> {
    Success(T),

    FailedToExecute,
    Unsupported,
    InvalidCall,
    PreChecksFailed,
}

impl<T> BiosCallResult<T> {
    pub unsafe fn ignore_error(self) -> Nothing {
        Nothing::default()
    }
    pub fn did_succeed(&self) -> bool {
        matches!(self, Self::Success(_))
    }
    pub fn did_fail(&self) -> bool {
        !self.did_succeed()
    }
    pub fn unwrap(self) -> T {
        match self {
            Success(value) => value,
            _ => panic!("BiosCall failed on 'unwrap'!"),
        }
    }
}

fn test() {
    let test: Option<()> = Option::None;

    test.unwrap()
}

#[allow(dead_code)]
pub struct BiosCall<CallType = NoCall> {
    registers_16: GPRegs16,
    registers_32: GPRegs32,
    segment_registers: SegmentRegs,
    reserved: PhantomData<CallType>,
}

impl BiosCall {
    const BDA_PTR: *const u8 = 0x0400 as *const u8;

    pub fn new() -> BiosCall<NoCall> {
        BiosCall {
            registers_16: Default::default(),
            registers_32: Default::default(),
            segment_registers: Default::default(),
            reserved: Default::default(),
        }
    }

    pub fn get_serial_io_ports() -> [u16; 4] {
        let buff_ptr = Self::BDA_PTR as *const [u16; 4];

        let read_data = unsafe { *buff_ptr };

        read_data
    }

    pub fn get_parallel_io_ports() -> [u16; 3] {
        let buff_ptr = unsafe { Self::BDA_PTR.add(0x8) } as *const [u16; 3];

        let read_data = unsafe { *buff_ptr };

        read_data
    }

    pub fn get_timer_ticks_since_midnight() -> u16 {
        let buff_ptr = unsafe { Self::BDA_PTR.add(0x6C) } as *const u16;

        let read_data = unsafe { *buff_ptr };

        read_data
    }

    
}

impl BiosCall<NoCall> {
    pub fn bit16_call(self) -> BiosCall<Bit16> {
        BiosCall {
            registers_16: Default::default(),
            registers_32: Default::default(),
            segment_registers: Default::default(),
            reserved: Default::default(),
        }
    }

    pub fn bit32_call(self) -> BiosCall<Bit32> {
        BiosCall {
            registers_16: Default::default(),
            registers_32: Default::default(),
            segment_registers: Default::default(),
            reserved: Default::default(),
        }
    }
}

impl BiosCall<Bit32> {
    #[inline(never)]
    unsafe fn general_purpose_memory_call(&mut self) {
        asm!(
        "mov es, {es}",
        "int 0x15",
        inout("eax") self.registers_32.eax => self.registers_32.eax,
        inout("ebx") self.registers_32.ebx => self.registers_32.ebx,
        inout("ecx") self.registers_32.ecx => self.registers_32.ecx,
        inout("edx") self.registers_32.edx => self.registers_32.edx,
        inout("edi") self.registers_32.edi => self.registers_32.edi,
        es = in(reg) self.segment_registers.es,
        );
    }

    pub unsafe fn memory_detection_operating(mut self, memory_entry_ptr: *const u8, ebx: u32) -> BiosCallResult<u32> {
        self.registers_32.ebx = ebx;
        self.registers_32.edi = memory_entry_ptr as u32;
        self.registers_32.edx = 0x534D4150;
        self.registers_32.eax = 0xE820;
        self.registers_32.ecx = 24;

        self.general_purpose_memory_call();
        let carry_flag = EFLAGS::is_carry_flag_set();

        if carry_flag {
            return BiosCallResult::FailedToExecute;
        }
        if (self.registers_16.ax & 0xFF00 >> 8) == 0x80 {
            return BiosCallResult::InvalidCall;
        }
        if (self.registers_16.ax & 0xFF00 >> 8) == 0x86 {
            return BiosCallResult::Unsupported;
        }

        BiosCallResult::Success(self.registers_32.ebx)
    }
}

impl BiosCall<Bit16> {
    #[inline(never)]
    unsafe fn general_purpose_video_call(&mut self) {
        asm!(
        "int 0x10",
        inout("ax") self.registers_16.ax => self.registers_16.ax,
        inout("bx") self.registers_16.bx => self.registers_16.bx,
        inout("cx") self.registers_16.cx => self.registers_16.cx,
        inout("dx") self.registers_16.dx => self.registers_16.dx,
        inout("di") self.registers_16.di => self.registers_16.di,
        );
    }

    #[inline(never)]
    unsafe fn general_purpose_disk_call(&mut self) {
        asm!(
        "push si",
        "mov si, {si_imp:x}",
        "int 0x13",
        "pop si",
        si_imp = in(reg) self.registers_16.si,
        inout("ax") self.registers_16.ax => self.registers_16.ax,
        inout("bx") self.registers_16.bx => self.registers_16.bx,
        inout("cx") self.registers_16.cx => self.registers_16.cx,
        inout("dx") self.registers_16.dx => self.registers_16.dx,
        );
    }

    pub unsafe fn display_char(mut self, c: char, attributes: u16) -> BiosCallResult<Nothing> {
        if !c.is_ascii() {
            return PreChecksFailed;
        }

        if !((c as u16) & 0x00FF == c as u16) {
            return PreChecksFailed;
        }

        self.registers_16.ax = 0x0E00 | (c as u16 & 0x00FF);
        self.registers_16.cx = 0x1;
        self.registers_16.bx = attributes;

        self.general_purpose_video_call();
        let carry_flag = EFLAGS::is_carry_flag_set();

        if carry_flag {
            return BiosCallResult::FailedToExecute;
        }
        if (self.registers_16.ax & 0xFF00 >> 8) == 0x80 {
            return BiosCallResult::InvalidCall;
        }
        if (self.registers_16.ax & 0xFF00 >> 8) == 0x86 {
            return BiosCallResult::Unsupported;
        }

        BiosCallResult::Success(Nothing::default())
    }

    pub unsafe fn bios_disk_io(
        mut self,
        disk_id: u8,
        disk_packet: *mut u8,
        is_writting: bool,
    ) -> BiosCallResult<Nothing> {
        if disk_packet as u32 == 0 {
            return PreChecksFailed;
        }
        if *disk_packet != 12 && *disk_packet != 16 {
            return PreChecksFailed;
        }

        self.registers_16.ax = if is_writting { 0x4300 } else { 0x4200 };
        self.registers_16.dx = disk_id as u16;
        self.registers_16.si = disk_packet as u16;

        self.general_purpose_disk_call();
        let carry_flag = EFLAGS::is_carry_flag_set();

        if carry_flag {
            return BiosCallResult::FailedToExecute;
        }
        if (self.registers_16.ax & 0xFF00 >> 8) == 0x80 {
            return BiosCallResult::InvalidCall;
        }
        if (self.registers_16.ax & 0xFF00 >> 8) == 0x86 {
            return BiosCallResult::Unsupported;
        }

        BiosCallResult::Success(Nothing::default())
    }

    pub unsafe fn read_vbe_info(mut self, struct_ptr: *mut u8) -> BiosCallResult<Nothing> {
        assert_ne!(struct_ptr as u16, 0);
        assert!(struct_ptr as u16 <= u16::MAX);

        self.registers_16.di = struct_ptr as u16;
        self.registers_16.ax = 0x4F00;

        self.general_purpose_video_call();
        let carry_flag = EFLAGS::is_carry_flag_set();

        if carry_flag {
            return BiosCallResult::FailedToExecute;
        }
        if (self.registers_16.ax & 0xFF00 >> 8) == 0x80 {
            return BiosCallResult::InvalidCall;
        }
        if (self.registers_16.ax & 0xFF00 >> 8) == 0x86 {
            return BiosCallResult::Unsupported;
        }

        BiosCallResult::Success(Nothing::default())
    }

    pub unsafe fn read_vbe_mode(
        mut self,
        struct_ptr: *mut u8,
        mode: u16,
    ) -> BiosCallResult<Nothing> {
        assert_ne!(struct_ptr as u16, 0);
        assert!(struct_ptr as u16 <= u16::MAX);
        assert_ne!(mode, 0);

        self.registers_16.di = struct_ptr as u16;
        self.registers_16.cx = mode;
        self.registers_16.ax = 0x4F01;

        self.general_purpose_video_call();
        let carry_flag = EFLAGS::is_carry_flag_set();

        if carry_flag {
            return BiosCallResult::FailedToExecute;
        }
        if (self.registers_16.ax & 0xFF00 >> 8) == 0x80 {
            return BiosCallResult::InvalidCall;
        }
        if (self.registers_16.ax & 0xFF00 >> 8) == 0x86 {
            return BiosCallResult::Unsupported;
        }

        BiosCallResult::Success(Nothing::default())
    }

    pub unsafe fn set_vbe_mode(mut self, mode: u16) -> BiosCallResult<Nothing> {
        assert_ne!(mode, 0);

        self.registers_16.ax = 0x4F02;
        self.registers_16.bx = mode;

        self.general_purpose_video_call();

        let carry_flag = EFLAGS::is_carry_flag_set();

        if carry_flag {
            return BiosCallResult::FailedToExecute;
        }
        if (self.registers_16.ax & 0xFF00 >> 8) == 0x80 {
            return BiosCallResult::InvalidCall;
        }
        if (self.registers_16.ax & 0xFF00 >> 8) == 0x86 {
            return BiosCallResult::Unsupported;
        }

        BiosCallResult::Success(Nothing::default())
    }
}
