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
    Com8
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
    Baud300
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
        let io_ports = BiosCall::get_serial_io_ports();

         match self {
            Self::Com1 => Some(io_ports[0]),
            Self::Com2 => Some(io_ports[1]),
            Self::Com3 => Some(io_ports[2]),
            Self::Com4 => Some(io_ports[3]),
            _ => None
        }
    }
}

pub struct SerialDevice {
    port_id: IOPort,
    configured_speed: u32
}

pub enum SerialErr {
    SerialPortUnknown,
}

enum LineStatusRegister {
    DataReady,
    OverrunErr,
    ParityErr,
    FramingErr,
    BreakErr,
    TransmitterHoldingRegisterEmpty,
    TransmitterEmpty,
    ImpendingErr
}

/*
Bit	Name	Meaning
0	Data ready (DR)	Set if there is data that can be read
1	Overrun error (OE)	Set if there has been data lost
2	Parity error (PE)	Set if there was an error in the transmission as detected by parity
3	Framing error (FE)	Set if a stop bit was missing
4	Break indicator (BI)	Set if there is a break in data input
5	Transmitter holding register empty (THRE)	Set if the transmission buffer is empty (i.e. data can be sent)
6	Transmitter empty (TEMT)	Set if the transmitter is not doing anything
7	Impending Error	Set if there is an error with a word in the input buffer
*/

enum ModemStatusRegister {
    DeltaClearToSend,
    DeltaDataSetReady,
    TrailingEdgeOfRingIndicator,
    DeltaDataCarrierDetect,
    ClearToSend,
    DataSetReady,
    RingIndicator,
    DataCarrierDetect
}

/*
Modem Status Register
This register provides the current state of the control lines from a peripheral device. In addition to this current-state information, four bits of the MODEM Status Register provide change information. These bits are set to a logic 1 whenever a control input from the MODEM changes state. They are reset to logic 0 whenever the CPU reads the MODEM Status Register

Bit	Name	Meaning
0	Delta Clear to Send (DCTS)	Indicates that CTS input has changed state since the last time it was read
1	Delta Data Set Ready (DDSR)	Indicates that DSR input has changed state since the last time it was read
2	Trailing Edge of Ring Indicator (TERI)	Indicates that RI input to the chip has changed from a low to a high state
3	Delta Data Carrier Detect (DDCD)	Indicates that DCD input has changed state since the last time it ware read
4	Clear to Send (CTS)	Inverted CTS Signal
5	Data Set Ready (DSR)	Inverted DSR Signal
6	Ring Indicator (RI)	Inverted RI Signal
7	Data Carrier Detect (DCD)	Inverted DCD Signal
*/

impl SerialDevice {


    pub unsafe fn from_raw(port: IOPort, speed: u16) -> Result<Self, SerialErr> {



    }

    pub fn new(port: SerialCom, baud: SerialBaud) -> Result<Self, SerialErr> {
        let port = match port {
            Some(value) => value,
            None => return Err(SerialErr::SerialPortUnknown)
        };

        unsafe {
            Self::from_raw(port, baud.get_divisor())
        }
    }
}