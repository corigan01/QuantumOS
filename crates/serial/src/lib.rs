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

use arch::io::IOPort;

pub mod baud;
mod registers;

pub struct Serial {
    baud: baud::SerialBaud,
    port: IOPort,
}

/// # Init Serial Device
/// Probe and init a serial device.
unsafe fn init_serial_device(baud: baud::SerialBaud, port: IOPort) -> bool {
    // Disable interrupts and enable DLAB bit
    registers::write_interrupt_enable(port, 0x00);
    registers::write_line_control(port, 0x80);

    // Setup devisor and loopback
    let divisor = baud.get_divisor();
    registers::write_dlab_lsb(port, divisor as u8);
    registers::write_dlab_msb(port, (divisor >> 8) as u8);
    registers::write_line_control(port, 0x03); /* 8-bit char, no parity mode, and one stop bit */
    registers::write_fifo_control(port, 0xC7);
    registers::write_modem_control(port, 0x0B);
    registers::write_modem_control(port, 0x1E);

    // Preform 3 tests to check if loopback is working
    registers::write_transmit_buffer(port, 0xFF);
    if registers::read_receive_buffer(port) != 0xFF {
        return false;
    }

    registers::write_transmit_buffer(port, 0xAB);
    if registers::read_receive_buffer(port) != 0xAB {
        return false;
    }

    registers::write_transmit_buffer(port, 0x00);
    if registers::read_receive_buffer(port) != 0x00 {
        return false;
    }

    // Finally turn off loopback and go into normal mode
    registers::write_modem_control(port, 0x0F);

    true
}

impl Serial {
    /// # Probe First
    /// Probe for the first com port that will hold and loop-back data.
    ///
    /// (When using an Emulator this is the best option to find which
    ///  serial port the emulator is connected to.)
    pub fn probe_first(baud: baud::SerialBaud) -> Option<Self> {
        for port in registers::ports::COMMS_ARRAY {
            if unsafe { init_serial_device(baud, port) } {
                return Some(Self { baud, port });
            }
        }

        None
    }

    /// # Transmit Byte
    /// This will send a byte over serial.
    #[inline]
    pub fn transmit_byte(&self, byte: u8) {
        unsafe { registers::write_transmit_buffer(self.port, byte) };
    }

    /// # Get Baud
    /// Get the currently set baud rate.
    pub fn get_baud(&self) -> baud::SerialBaud {
        self.baud
    }
}

impl core::fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.transmit_byte(byte);
        }

        Ok(())
    }
}
