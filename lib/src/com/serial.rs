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
use crate::enum_flags;
use crate::x86_64::bios_call::BiosCall;
use crate::x86_64::io::port::IOPort;

pub enum SerialCom {
    Com1,
    Com2,
    Com3,
    Com4,
    Com5,
    Com6,
    Com7,
    Com8,
}

pub enum SerialBaud {
    Baud115200,
    Baud57600,
    Baud38400,
    Baud19200,
    Baud14400,
    Baud9600,
    Baud4800,
    Baud2400,
    Baud1200,
    Baud600,
    Baud300,
}

impl SerialBaud {
    pub fn get_divisor(&self) -> u16 {
        match self {
            SerialBaud::Baud115200 => 1,
            SerialBaud::Baud57600 => 2,
            SerialBaud::Baud38400 => 3,
            SerialBaud::Baud19200 => 6,
            SerialBaud::Baud14400 => 8,
            SerialBaud::Baud9600 => 12,
            SerialBaud::Baud4800 => 24,
            SerialBaud::Baud2400 => 48,
            SerialBaud::Baud1200 => 96,
            SerialBaud::Baud600 => 192,
            SerialBaud::Baud300 => 348,
        }
    }
}

impl SerialCom {
    pub fn get_port_id_from_bios(&self) -> Option<IOPort> {
        // FIXME: Allow for more ways of getting IO PORT ID?
        let io_ports = BiosCall::get_serial_io_ports();

        match self {
            Self::Com1 => Some(io_ports[0]),
            Self::Com2 => Some(io_ports[1]),
            Self::Com3 => Some(io_ports[2]),
            Self::Com4 => Some(io_ports[3]),
            _ => None,
        }
    }
}

pub enum SerialErr {
    SerialPortUnknown,
}

pub struct SerialDevice {
    port_id: IOPort,
    configured_speed: u32,
}

impl SerialDevice {
    pub fn disable_interrupts(&self) {
        const DISABLE_INT_OFFSET: i16 = 1;

        unsafe {
            self.port_id
                .clone_from_offset_by(DISABLE_INT_OFFSET)
                .write_u8(0);
        }
    }

    pub unsafe fn from_raw(port: IOPort, speed: u16) -> Result<Self, SerialErr> {
        todo!()
    }

    pub fn new(port: SerialCom, baud: SerialBaud) -> Result<Self, SerialErr> {
        let port = match port.get_port_id_from_bios() {
            Some(value) => value,
            None => return Err(SerialErr::SerialPortUnknown),
        };

        unsafe { Self::from_raw(port, baud.get_divisor()) }
    }
}

enum_flags! {
    #[derive(Clone, Copy, PartialEq)]
    enum LineStatusRegister(LineStatusRegisterFlags) {
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

    fn read_register(mut port: IOPort) -> LineStatusRegisterFlags {
        port.mutate_offset_by(Self::LINE_STATUS_REGISTER_OFFSET);

        let value = unsafe { port.read_u8() };

        unsafe { LineStatusRegisterFlags::unsafe_from_value(value as usize) }
    }
}

enum_flags! {
    #[derive(Clone, Copy, PartialEq)]
    enum ModemStatusRegister(ModemStatusRegisterFlags) {
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

    fn read_register(mut port: IOPort) -> ModemStatusRegisterFlags {
        port.mutate_offset_by(Self::MODEM_STATUS_REGISTER_OFFSET);

        let value = unsafe { port.read_u8() };

        unsafe { ModemStatusRegisterFlags::unsafe_from_value(value as usize) }
    }
}
