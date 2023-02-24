/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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

use crate::port;
use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
#[allow(dead_code)]
pub enum SerialCOM {
    Com1 = 0x3F8,
    Com2 = 0x2F8,
    Com3 = 0x3E8,
    Com4 = 0x2E8,
    None = 0x00
}

impl SerialCOM {
    fn is_none(&self) -> bool {
        *self == Self::None
    }

    fn unwrap(&self) -> Self {
        if self.is_none() {
            panic!("SerialCOM is \'None\'");
        }

        *self
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaudRate {
    Baud115200 = 1,
    Baud57600  = 2,
    Baud38400  = 3,
    Baud19200  = 6,
    Baud14400  = 8,
    Baud9600   = 12,
    Baud4800   = 24,
    Baud2400   = 48,
    Baud1200   = 96,
    Baud600    = 192,
    Baud300    = 384
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SerialDevice {
    port: SerialCOM,
    baud: BaudRate
}

impl SerialDevice {

    // Constructs a new SerialDevice
    // This function will return a `Option<T>` in the event that
    pub fn new(port: SerialCOM, baud: BaudRate) -> Option<SerialDevice> {
        let mode = port as u16;

        unsafe {
            /*
                | IO |  D  |  	Register mapped to this port
                |----|-----|----------------------------------------------------
                | +0 |	0  |	Data register. Reading this registers read from the Receive buffer. Writing to this register writes to the Transmit buffer.
                | +1 |	0  |	Interrupt Enable Register.
                | +0 |	1  |	With DLAB set to 1, this is the least significant byte of the divisor value for setting the baud rate.
                | +1 |	1  |	With DLAB set to 1, this is the most significant byte of the divisor value.
                | +2 |	-  |	Interrupt Identification and FIFO control registers
                | +3 |	-  |	Line Control Register. The most significant bit of this register is the DLAB.
                | +4 |	-  |	Modem Control Register.
                | +5 |	-  |	Line Status Register.
                | +6 |	-  |	Modem Status Register.
                | +7 |	-  |	Scratch Register.

                (https://wiki.osdev.org/Serial_Ports)
            */

            // INIT
            port::byte_out(mode + 1, 0x00); // Disable the Serial Port interrupts
            port::byte_out(mode + 3, 0x80); // Enable DLAB which sets the baud rate divisor

            let baud_low = baud as u8;
            let baud_high = ((baud as u16) >> 8) as u8;

            port::byte_out(mode + 0, baud_low);  // Set the baud rate         (lo byte)
            port::byte_out(mode + 1, baud_high); //                           (hi byte)

            port::byte_out(mode + 3, 0x03); // Use 8 bits, no parity bits, and one stop bit
            port::byte_out(mode + 2, 0xC7); // Enable FIFO

            // TEST the port first
            let test_byte: u8 = b'A';

            port::byte_out(mode + 4, 0x1E); // Set Serial to loopback mode
            port::byte_out(mode + 0, test_byte); // Send test byte

            let serial_response = port::byte_in(mode + 0);

            if serial_response != test_byte {
                return None;
            }

            // Success
            port::byte_out(mode + 4, 0x0F); // Set Serial to normal mode
        }

        Some(SerialDevice { port , baud })
    }

    unsafe fn is_transmit_empty(&self) -> bool {
        port::byte_in(self.port as u16 + 5) & 0x20 != 0x00
    }

    pub fn get_baud_div(&self) -> BaudRate {
        self.baud.clone()
    }

    pub fn get_baud(&self) -> u32 {
        115200 / (self.get_baud_div() as u32)
    }

    pub fn write_byte(&self, byte: u8) {
        let port = self.port.unwrap() as u16;

        unsafe {
            loop {
              if self.is_transmit_empty() {
                  break;
              }
            }

            port::byte_out(port, byte);
        }
    }

    pub fn write_string(&self, string: &str) {
        for byte in string.bytes() {
            match byte {
                _ => self.write_byte(byte)
            }
        }
    }
}

impl fmt::Write for SerialDevice {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref SERIAL1: Mutex<Option<SerialDevice>> =
        Mutex::new(SerialDevice::new(SerialCOM::Com1, BaudRate::Baud115200));
}


#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    if SERIAL1.lock().as_ref() != None {
        SERIAL1.lock().unwrap().write_fmt(args).unwrap();
    }
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::debug_print!("\n"));
    ($fmt:expr) => ($crate::debug_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::debug_print!(
        concat!($fmt, "\n"), $($arg)*));
}