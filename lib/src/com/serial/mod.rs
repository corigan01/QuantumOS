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

pub use crate::com::serial::options::*;
use crate::com::serial::registers::*;

use crate::x86_64::io::port::IOPort;

pub mod options;
pub mod registers;

pub struct SerialDevice {
    port: SerialPort,
    io_port: IOPort,
    baud: SerialBaud,

    // Serial Registers
    int_reg: InterruptEnableRegister,
    line_reg: LineControlRegister,
    mod_reg: ModemControlRegister,
}

impl SerialDevice {
    pub fn new(port: SerialPort, baud: SerialBaud) -> Result<Self, SerialErr> {
        let baud_dev = baud.get_divisor();
        let port_id = match port.get_port_id_from_bios() {
            Some(value) => value,
            None => return Err(SerialErr::UnknownPort),
        };

        if port_id.as_u16() == 0 {
            return Err(SerialErr::PortNotConnected);
        }

        let line_register = LineControlRegister::new(port_id);
        let interrupt_register = InterruptEnableRegister::new(port_id);
        let modem_register = ModemControlRegister::new(port_id);

        interrupt_register.turn_off_interrupts();

        line_register.set_divisor_using_dlab(baud_dev);
        line_register.set_parity(SerialParity::None);
        line_register.set_data_bits(SerialDataBits::CharacterLen8);
        line_register.set_stop_bits(SerialStopBits::StopBits1);

        modem_register.loopback_mode(true);

        let test_byte = 0xAE;

        unsafe { port_id.write_u8(test_byte) };
        let in_byte = unsafe { port_id.read_u8() };

        if test_byte != in_byte {
            return Err(SerialErr::InitFailed);
        }

        modem_register.loopback_mode(false);

        Ok(Self {
            port,
            io_port: port_id,
            baud,
            int_reg: interrupt_register,
            line_reg: line_register,
            mod_reg: modem_register,
        })
    }

    pub fn send_byte_sync(&self, byte: u8) {
        loop {
            let flags = LineStatusRegister::read_register(self.io_port);
            let flag_active = flags.is_flag_active(LineStatusRegister::TransmitterEmpty);

            if flag_active {
                break;
            }
        }

        unsafe { self.io_port.write_u8(byte) };
    }

    pub fn recv_byte_sync(&self) -> Option<u8> {
        let flags = LineStatusRegister::read_register(self.io_port);
        let is_data_ready = flags.is_flag_active(LineStatusRegister::DataReady);

        if is_data_ready {
            Some(unsafe { self.io_port.read_u8() })
        } else {
            None
        }
    }
}
