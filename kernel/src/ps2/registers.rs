/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

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

use quantum_lib::x86_64::io::port::IOPort;
use quantum_utils::bitset::BitSet;

/// # Read Register
/// Given the constant of the port, it will construct
/// fuctions to read and read_u16 from the given port.
///
/// This trait is unsafe because the programmer must
/// ensure that this port has no side effects when
/// preforming read operations, because they are
/// considered safe inside the trait.
pub unsafe trait ReadRegister<const REGISTER_ID: u16> {
    /// # Read
    /// Read a u8 from the Register.
    fn read() -> u8 {
        unsafe { IOPort::new(REGISTER_ID).read_u8() }
    }

    /// # Read u16
    /// Read a u16 from the Register.
    fn read_u16() -> u16 {
        unsafe { IOPort::new(REGISTER_ID).read_u16() }
    }
}

/// # Write Register
/// Given the constant of the port, it will construct
/// functions to write and write_u16 to the given port.
///
/// This trait is unsafe because the programmer must
/// ensure that this IOPort is valid for writting.
pub unsafe trait WriteRegister<const REGISTER_ID: u16> {
    /// # Write
    /// Write a u8 to the Register.
    unsafe fn write(value: u8) {
        IOPort::new(REGISTER_ID).write_u8(value)
    }

    /// # Write u16
    /// Write a u16 to the Register.
    unsafe fn write_u16(value: u16) {
        IOPort::new(REGISTER_ID).write_u16(value)
    }
}

/// Data Register IO Port
const DATA_REGISTER_PORT_OFFSET: u16 = 0x60;
/// Status Register IO Port
const STATUS_REGISTER_PORT_OFFSET: u16 = 0x64;
/// Command Register IO Port
const COMMAND_REGISTER_PORT_OFFSET: u16 = 0x64;

/// # Data Register
/// Used to control the PS2 Device and Controller to send and
/// recv data.
pub struct DataRegister {}
unsafe impl ReadRegister<DATA_REGISTER_PORT_OFFSET> for DataRegister {}
unsafe impl WriteRegister<DATA_REGISTER_PORT_OFFSET> for DataRegister {}

/// # Status Flags
/// Possible status flags for the status register in the ps2 controller.
/// Flags can be read using the StatusFieldFlags struct.
pub enum StatusFlags {
    /// # Output Buffer Full
    /// The output buffer status is full. Must be full before reading
    /// from the Data Register.
    OutputBufferFull,
    /// # Output Buffer Empty
    /// The output buffer status is not full. You must wait before
    /// reading the Data Register.
    OutputBufferEmpty,
    /// # Input Buffer Full
    /// The input buffer is full. You must wait before writing to
    /// Data Register or Command Register.
    InputBufferFull,
    /// # Input Buffer Empty
    /// The input buffer is not full. Must be empty before writting
    /// to the Data Register or Command Register.
    InputBufferEmpty,
    /// # System Flag
    /// The system flag is used to indicate if the firmware passes self
    /// tests. This should always be set!
    SystemFlag,
    /// # Device Targeted Data
    /// Data is going to be written to the PS2 Device instead of the
    /// controller.
    DeviceTargetedData,
    /// # Controller Targeted Data
    /// Data is going to be written to the PS2 Controller instead of
    /// the device.
    ControllerTargetedData,
    /// # Timeout Error
    /// The controller has timed out.
    TimeoutError,
    /// # Parity Error
    /// The controller has detected a parity error with the connected
    /// device.
    ParityError,
}

/// # Status Field Flags
/// Stores the flags read from the register for reading. Use the
/// StatusFlags enum to read from this struct.
///
/// This struct can only be constructed from the StatusRegister.
pub struct StatusFieldFlags(u8);

impl StatusFieldFlags {
    // --- Bit offsets
    const OUTPUT_BUFFER_STATUS_BIT: u8 = 0;
    const INPUT_BUFFER_STATUS_BIT: u8 = 1;
    const SYSTEM_FLAG_STATUS_BIT: u8 = 2;
    const COMMAND_DATA_STATUS_BIT: u8 = 3;
    const TIME_OUT_ERROR_STATUS_BIT: u8 = 6;
    const PARITY_ERROR_STATUS_BIT: u8 = 7;

    /// # Check Flag
    /// Used to check if a StatusFlag is active or inactive. Not all StatusFlags
    /// are real bits, so this is used to help abstract the bit states from the
    /// programmer to avoid confusion.
    #[inline]
    pub fn check_flag(&self, flag: StatusFlags) -> bool {
        let flags = *self.0;
        match flag {
            StatusFlags::OutputBufferFull => flags.get_bit(Self::OUTPUT_BUFFER_STATUS_BIT),
            StatusFlags::OutputBufferEmpty => !flags.get_bit(Self::OUTPUT_BUFFER_STATUS_BIT),
            StatusFlags::InputBufferFull => flags.get_bit(Self::INPUT_BUFFER_STATUS_BIT),
            StatusFlags::InputBufferEmpty => !flags.get_bit(Self::INPUT_BUFFER_STATUS_BIT),
            StatusFlags::SystemFlag => flags.get_bit(Self::SYSTEM_FLAG_STATUS_BIT),
            StatusFlags::DeviceTargetedData => !flags.get_bit(Self::COMMAND_DATA_STATUS_BIT),
            StatusFlags::ControllerTargetedData => flags.get_bit(Self::COMMAND_DATA_STATUS_BIT),
            StatusFlags::TimeoutError => flags.get_bit(Self::TIME_OUT_ERROR_STATUS_BIT),
            StatusFlags::ParityError => flags.get_bit(Self::PARITY_ERROR_STATUS_BIT),
        }
    }
}

/// # Status Register
/// The status register contains flags about the state of the PS2 controller.
pub struct StatusRegister {}
unsafe impl ReadRegister<STATUS_REGISTER_PORT_OFFSET> for StatusRegister {}

impl StatusRegister {
    /// # Get Status
    /// Get the StatusFieldFlags status of the controller.
    pub fn get_status() -> StatusFieldFlags {
        StatusFieldFlags(Self::read())
    }
}

pub struct CommandRegister {}
unsafe impl WriteRegister<COMMAND_REGISTER_PORT_OFFSET> for CommandRegister {}
