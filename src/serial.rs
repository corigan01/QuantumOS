use crate::port;
use crate::port::{byte_in, byte_out};
use crate::serial::SerialCOM::COM_1;
use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt;
use core::fmt::Arguments;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
#[allow(dead_code)]
pub enum SerialCOM {
    COM_1 = 0x3F8,
    COM_2 = 0x2F8,
    COM_3 = 0x3E8,
    COM_4 = 0x2E8,
    COM_NULL = 0x00
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SerialDevice {
    port: SerialCOM
}

impl SerialDevice {

    // Constructs a new SerialDevice
    // This function will return a `Option<T>` in the event that
    pub fn new(port: SerialCOM) -> Option<SerialDevice> {
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
            port::byte_out(mode + 0, 0x03); // Set the baud rate to 38400 (lo byte)
            port::byte_out(mode + 1, 0x00); //                            (hi byte)
            port::byte_out(mode + 3, 0x03); // Use 8 bits, no parity bits, and one stop bit
            port::byte_out(mode + 2, 0xC7); // Enable FIFO
            port::byte_out(mode + 4, 0x1E); // Set Serial to loopback mode

            // TEST
            let test_byte: u8 = b'A';

            port::byte_out(mode + 0, test_byte); // Send test byte

            let serial_response = port::byte_in(mode + 0);

            if serial_response != test_byte {
                return None;
            }

            // Success
            port::byte_out(mode + 4, 0x0F);
        }

        Some(SerialDevice { port })

    }

    unsafe fn is_transmit_empty(&self) -> bool {
        port::byte_in(self.port as u16 + 5) & 0x20 != 0x00
    }

    pub fn write_byte(&self, byte: u8) {
        assert_ne!(self.port, SerialCOM::COM_NULL);

        unsafe {
            loop {
              if self.is_transmit_empty() {
                  break;
              }
            }

            port::byte_out(self.port as u16, byte);
        }
    }

    pub fn write_string(&self, string: &str) {
        for byte in string.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe)
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
        Mutex::new(SerialDevice::new(SerialCOM::COM_2));
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
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}