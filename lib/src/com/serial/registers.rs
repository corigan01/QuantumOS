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

use crate::bitset::BitSet;
use crate::enum_iterator::EnumIntoIterator;
use crate::x86_64::io::port::IOPort;
use crate::{enum_flags, enum_iterator};

#[derive(Debug, Copy, Clone)]
pub enum SerialErr {
    UnknownPort,
    InitFailed,
    PortNotConnected,
}

enum_flags! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum LineStatusRegister(LineStatusRegisterFlags) {
        DataReady,
        OverrunErr,
        ParityErr,
        FramingErr,
        BreakErr,
        TransmitterHoldingRegisterEmpty,
        TransmitterEmpty,
        ImpendingErr
    }
}

impl LineStatusRegister {
    const LINE_STATUS_REGISTER_OFFSET: i16 = 5;

    pub fn read_register(port: IOPort) -> LineStatusRegisterFlags {
        let port = port.clone_from_offset_by(Self::LINE_STATUS_REGISTER_OFFSET);

        let value = unsafe { port.read_u8() };

        unsafe { LineStatusRegisterFlags::unsafe_from_value(value as usize) }
    }
}

enum_flags! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum ModemStatusRegister(ModemStatusRegisterFlags) {
        DeltaClearToSend,
        DeltaDataSetReady,
        TrailingEdgeOfRingIndicator,
        DeltaDataCarrierDetect,
        ClearToSend,
        DataSetReady,
        RingIndicator,
        DataCarrierDetect
    }
}

impl ModemStatusRegister {
    const MODEM_STATUS_REGISTER_OFFSET: i16 = 6;

    pub fn read_register(port: IOPort) -> ModemStatusRegisterFlags {
        let port = port.clone_from_offset_by(Self::MODEM_STATUS_REGISTER_OFFSET);

        let value = unsafe { port.read_u8() };

        unsafe { ModemStatusRegisterFlags::unsafe_from_value(value as usize) }
    }
}

pub struct ModemControlRegister {
    port: IOPort,
}

impl ModemControlRegister {
    pub fn new(port: IOPort) -> Self {
        Self {
            port: port.clone_from_offset_by(4),
        }
    }

    fn control_flag(&self, pin: u8, flag: bool) {
        let mut last_value = unsafe { self.port.read_u8() };
        last_value.set_bit(pin, flag);

        unsafe {
            self.port.write_u8(last_value);
        }
    }

    pub fn data_terminal_read(&self, flag: bool) {
        self.control_flag(0, flag);
    }

    pub fn request_to_send(&self, flag: bool) {
        self.control_flag(1, flag);
    }

    pub fn out_1(&self, flag: bool) {
        self.control_flag(2, flag);
    }

    pub fn out_2(&self, flag: bool) {
        self.control_flag(3, flag);
    }

    pub fn loopback_mode(&self, flag: bool) {
        self.control_flag(4, flag)
    }
}

pub struct InterruptEnableRegister {
    port: IOPort,
}

impl InterruptEnableRegister {
    pub fn new(port: IOPort) -> Self {
        Self {
            port: port.clone_from_offset_by(1),
        }
    }

    fn control_flag(&self, pin: u8, flag: bool) {
        let mut last_value = unsafe { self.port.read_u8() };
        last_value.set_bit(pin, flag);

        unsafe {
            self.port.write_u8(last_value);
        }
    }

    pub fn set_data_available_interrupt(&self, flag: bool) {
        self.control_flag(0, flag);
    }

    pub fn set_transmitter_empty_interrupt(&self, flag: bool) {
        self.control_flag(1, flag);
    }

    pub fn set_on_error_interrupt(&self, flag: bool) {
        self.control_flag(2, flag);
    }

    pub fn set_on_status_change_interrupt(&self, flag: bool) {
        self.control_flag(3, flag);
    }

    pub fn turn_off_interrupts(&self) {
        unsafe {
            self.port.write_u8(0);
        }
    }
}

enum_iterator! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum SerialDataBits(SerialDataBitsIter) {
        CharacterLen5,
        CharacterLen6,
        CharacterLen7,
        CharacterLen8
    }
}

pub enum SerialStopBits {
    StopBits1,
    StopBits2,
}

enum_iterator! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum SerialParity(SerialParityIter) {
        None,
        Odd,
        Even,
        Mark,
        Space
    }
}

pub struct LineControlRegister {
    port: IOPort,
}

impl LineControlRegister {
    pub fn new(port: IOPort) -> Self {
        Self {
            port: port.clone_from_offset_by(3),
        }
    }

    fn control_flag(&self, pin: u8, flag: bool) {
        let mut last_value = unsafe { self.port.read_u8() };
        last_value.set_bit(pin, flag);

        unsafe {
            self.port.write_u8(last_value);
        }
    }

    pub fn set_divisor_using_dlab(&self, div: u16) {
        // enable dlab
        self.control_flag(7, true);

        let mut og_port = self.port.clone_from_offset_by(-3);

        unsafe {
            og_port.write_u8((div & 0x00FF) as u8);
        }
        og_port.mutate_offset_by(1);
        unsafe {
            og_port.write_u8(((div & 0xFF00) >> 8) as u8);
        }

        self.control_flag(7, false);
    }

    pub fn set_data_bits(&self, bits: SerialDataBits) {
        let index_of = SerialDataBits::get_index_of(bits);

        let data = unsafe { self.port.read_u8() }.set_bits(0..2, index_of as u64);

        unsafe { self.port.write_u8(data) };
    }

    pub fn set_stop_bits(&self, bits: SerialStopBits) {
        match bits {
            SerialStopBits::StopBits1 => self.control_flag(2, false),
            SerialStopBits::StopBits2 => self.control_flag(2, true),
        }
    }

    pub fn set_parity(&self, parity: SerialParity) {
        let index_of = SerialParity::get_index_of(parity);

        let data = unsafe { self.port.read_u8() }.set_bits(3..6, index_of as u64);

        unsafe { self.port.write_u8(data) };
    }
}
