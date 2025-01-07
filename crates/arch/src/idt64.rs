/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

use crate::{registers::Segment, CpuPrivilege};

#[derive(Clone, Copy, Debug)]
pub enum GateKind {
    TaskGate,
    InterruptGate16,
    TrapGate16,
    InterruptGate,
    TrapGate,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InterruptDescTable([GateDescriptor; 256]);

#[make_hw(
    field(RW, 0..=15, offset_1),
    field(RW, 16..=31, segment_selector),
    field(RW, 32..=34, pub ist),
    field(RW, 40..=43, gate_type),
    field(RW, 45..=46, dpl),
    field(RW, 47, pub present),
    field(RW, 48..=63, offset_2),
    field(RW, 64..=95, offset_3),
)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GateDescriptor(u128);

impl GateDescriptor {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn set_offset(&mut self, offset: u64) {
        self.set_offset_1((offset & (u16::MAX as u64)) as u16);
        self.set_offset_2(((offset >> 16) & (u16::MAX as u64)) as u16);
        self.set_offset_3(((offset >> 32) & (u32::MAX as u64)) as u32);
    }

    pub const fn get_offset(&self) -> u64 {
        (self.get_offset_1() as u64)
            | ((self.get_offset_2() as u64) << 16)
            | ((self.get_offset_3() as u64) << 32)
    }

    pub const fn set_gate_kind(&mut self, kind: GateKind) {
        let gate_num = match kind {
            GateKind::TaskGate => 0x5,
            GateKind::InterruptGate16 => 0x6,
            GateKind::TrapGate16 => 0x7,
            GateKind::InterruptGate => 0xE,
            GateKind::TrapGate => 0xF,
        };

        self.set_gate_type(gate_num);
    }

    pub const fn get_gate_kind(&self) -> GateKind {
        match self.get_gate_type() {
            0x5 => GateKind::TaskGate,
            0x6 => GateKind::InterruptGate16,
            0x7 => GateKind::TrapGate16,
            0xE => GateKind::InterruptGate,
            0xF => GateKind::TrapGate,
            _ => unreachable!(),
        }
    }

    pub const fn set_privilege(&mut self, dpl: CpuPrivilege) {
        let dpl_num = match dpl {
            CpuPrivilege::Ring0 => 0,
            CpuPrivilege::Ring1 => 1,
            CpuPrivilege::Ring2 => 2,
            CpuPrivilege::Ring3 => 3,
        };

        self.set_dpl(dpl_num);
    }

    pub const fn get_privilege(&self) -> CpuPrivilege {
        match self.get_dpl() {
            0 => CpuPrivilege::Ring0,
            1 => CpuPrivilege::Ring1,
            2 => CpuPrivilege::Ring2,
            3 => CpuPrivilege::Ring3,
            _ => unreachable!(),
        }
    }

    pub const fn set_code_segment(&mut self, segment: Segment) {
        self.set_segment_selector(segment.0);
    }

    pub const fn get_code_segment(&self) -> Segment {
        Segment(self.get_segment_selector())
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InterruptFrame {
    pub ip: u64,
    pub code_seg: u64,
    pub flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u16,
}

type RawHandlerFuncNe = extern "x86-interrupt" fn(InterruptFrame);
type RawHandlerFuncE = extern "x86-interrupt" fn(InterruptFrame, u64);

#[derive(Debug, Clone, Copy)]
pub enum TableSelector {
    Gdt,
    Idt,
    Ldt,
}

// This is part of the `nice` user facing interrupt info
#[derive(Debug, Clone, Copy)]
pub enum InterruptFlags {
    DivisionError,
    Debug,
    NonMaskableInterrupt,
    Breakpoint,
    Overflow,
    BoundRangeExceeded,
    InvalidOpcode,
    DeviceNotAvailable,
    DoubleFault,
    CoprocessorSegmentOverrun,
    InvalidTSS {
        external: bool,
        table: TableSelector,
        table_index: u16,
    },
    SegmentNotPresent {
        external: bool,
        table: TableSelector,
        table_index: u16,
    },
    StackSegmentFault {
        external: bool,
        table: TableSelector,
        table_index: u16,
    },
    // TODO: This one has an error code, but its complex
    GeneralProtectionFault,
    PageFault {
        present: bool,
        write: bool,
        user: bool,
        reserved_write: bool,
        instruction_fetch: bool,
        protection_key: bool,
        shadow_stack: bool,
        software_guard: bool,
        // filled by cr2
        virt_addr: u64,
    },
    X87FloatingPointException,
    AlignmentCheck,
    MachineCheck,
    SIMDFloatingPointException,
    VirtualizationException,
    ControlProtectionException,
    HypervisorInjectionException,
    VMMCommunicationException,
    SecurityException,
    Reserved,
    TripleFault,
    Irq(u8),
}

#[derive(Debug, Clone, Copy)]
pub enum ExceptionKind {
    Interrupt,
    Fault,
    Trap,
    Abort,
}

impl InterruptFlags {
    pub const fn exception_kind(&self) -> ExceptionKind {
        match self {
            Self::DivisionError => ExceptionKind::Fault,
            Self::Debug => ExceptionKind::Fault,
            Self::NonMaskableInterrupt => ExceptionKind::Interrupt,
            Self::Breakpoint => ExceptionKind::Trap,
            Self::Overflow => ExceptionKind::Trap,
            Self::BoundRangeExceeded => ExceptionKind::Fault,
            Self::InvalidOpcode => ExceptionKind::Fault,
            Self::DeviceNotAvailable => ExceptionKind::Fault,
            Self::DoubleFault => ExceptionKind::Abort,
            Self::CoprocessorSegmentOverrun => ExceptionKind::Fault,
            Self::InvalidTSS { .. } => ExceptionKind::Fault,
            Self::SegmentNotPresent { .. } => ExceptionKind::Fault,
            Self::StackSegmentFault { .. } => ExceptionKind::Fault,
            Self::GeneralProtectionFault => ExceptionKind::Fault,
            Self::PageFault { .. } => ExceptionKind::Fault,
            Self::X87FloatingPointException => ExceptionKind::Fault,
            Self::AlignmentCheck => ExceptionKind::Fault,
            Self::MachineCheck => ExceptionKind::Abort,
            Self::SIMDFloatingPointException => ExceptionKind::Fault,
            Self::VirtualizationException => ExceptionKind::Fault,
            Self::ControlProtectionException => ExceptionKind::Fault,
            Self::HypervisorInjectionException => ExceptionKind::Fault,
            Self::VMMCommunicationException => ExceptionKind::Fault,
            Self::SecurityException => ExceptionKind::Fault,
            Self::Reserved => ExceptionKind::Abort,
            Self::TripleFault => ExceptionKind::Abort,
            Self::Irq(_) => ExceptionKind::Interrupt,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InterruptInfo {
    frame: InterruptFrame,
    flags: InterruptFlags,
    gate_kind: GateKind,
}
