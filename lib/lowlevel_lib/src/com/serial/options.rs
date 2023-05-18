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

use crate::enum_iterator;
use crate::enum_iterator::EnumIntoIterator;
use crate::x86_64::bios_call::BiosCall;
use crate::x86_64::io::port::IOPort;

enum_iterator! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum SerialPort(SerialPortIter) {
        Com1,
        Com2,
        Com3,
        Com4,
        Com5,
        Com6,
        Com7,
        Com8
    }
}

#[derive(Clone, Copy, PartialEq)]
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

impl SerialPort {
    pub fn get_port_id_from_bios(&self) -> Option<IOPort> {
        let index = Self::get_index_of(*self);
        let ports_from_bios = BiosCall::get_serial_io_ports();

        Some(*ports_from_bios.get(index)?)
    }
}
