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
        let flags = self.0;
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

/// # Test Status
/// Possible Result of testing the PS2 controller or its ports.
#[derive(Debug, Clone, Copy)]
pub enum TestStatus {
    /// # Test Passed
    /// The test has passed.
    TestPassed,
    /// # Test Failed
    /// This is only for the controller, as the ports will
    /// set one of the below options instead.
    TestFailed,
    /// # Clock Stuck Low
    /// The port failed its test, and detected its clock
    /// was stuck low.
    ClockStuckLow,
    /// # Clock Stuck High
    /// The port failed its test, and detected its clock
    /// was stuck high.
    ClockStuckHigh,
    /// # Data Stuck Low
    /// The port failed its test, and detected its data
    /// line was stuck low.
    DataStuckLow,
    /// # Data Stuck High
    /// The port failed its test, and detected its data
    /// line was stuck high.
    DataStuckHigh,
}

/// # Command Register
/// The command register is used to send commands to the ps2 controller.
pub struct CommandRegister {}
unsafe impl WriteRegister<COMMAND_REGISTER_PORT_OFFSET> for CommandRegister {}

impl CommandRegister {
    // --- Command Bytes
    const PS2_COMMAND_BYTE_READ_BEGIN: u8 = 0x20;
    const PS2_COMMAND_BYTE_READ_END: u8 = 0x3F;
    const PS2_COMMAND_BYTE_WRITE_BEGIN: u8 = 0x60;
    const PS2_COMMAND_BYTE_WRITE_END: u8 = 0x7F;
    const PS2_COMMAND_BYTE_DISABLE_SECOND: u8 = 0xA7;
    const PS2_COMMAND_BYTE_ENABLE_SECOND: u8 = 0xA8;
    const PS2_COMMAND_BYTE_TEST_SECOND: u8 = 0xA9;
    const PS2_COMMAND_BYTE_TEST_CONTROLLER: u8 = 0xAA;
    const PS2_COMMAND_BYTE_TEST_FIRST: u8 = 0xAB;
    const PS2_COMMAND_BYTE_DISABLE_FIRST: u8 = 0xAD;
    const PS2_COMMAND_BYTE_ENABLE_FIRST: u8 = 0xAE;
    const PS2_COMMAND_BYTE_READ_INPUT_PORT: u8 = 0xC0;
    const PS2_COMMAND_BYTE_READ_OUTPUT_PORT: u8 = 0xD0;
    const PS2_COMMAND_BYTE_WRITE_OUTPUT_PORT: u8 = 0xD1;
    const PS2_COMMAND_BYTE_WRITE_FIRST_OUTPUT: u8 = 0xD2;
    const PS2_COMMAND_BYTE_WRITE_SECOND_OUTPUT: u8 = 0xD3;
    const PS2_COMMAND_BYTE_WRITE_SECOND_INPUT: u8 = 0xD4;

    /// # Wait Write
    /// Waits until the StatusRegister returns that the input
    /// buffer is empty. This is normally used to wait until
    /// the controller is ready to write.
    fn wait_write() {
        loop {
            let status = StatusRegister::get_status();

            if status.check_flag(StatusFlags::InputBufferEmpty) {
                break;
            }
        }
    }

    /// # Wait Read
    /// Waits until the StatusRegister returns that the output
    /// buffer is full. This is normally used to wait untill the
    /// controller is ready to read.
    fn wait_read() {
        loop {
            let status = StatusRegister::get_status();

            if status.check_flag(StatusFlags::OutputBufferFull) {
                break;
            }
        }
    }

    /// # Read Controller Ram
    /// Read from the controller's memory. The PS2 controller stores its
    /// Controller Configuration Byte at address 0.
    pub fn read_controller_ram(address: u8) -> u8 {
        assert!(
            address < 32,
            "Address ({address}) is out of bounds of the PS2 controller's memory. Range is only 0..32"
        );

        unsafe { Self::write(Self::PS2_COMMAND_BYTE_READ_BEGIN + address) };
        Self::wait_read();
        DataRegister::read()
    }

    /// # Write Controller Ram
    /// Write to the controller's memory. The PS2 controller stores its
    /// Controller Configuration Byte at address 0;
    pub unsafe fn write_controller_ram(address: u8, value: u8) {
        assert!(
            address < 32,
            "Address ({address}) is out of bounds of the PS2 controller's memory. Range is only 0..32"
        );

        Self::write(Self::PS2_COMMAND_BYTE_WRITE_BEGIN + address);
        Self::wait_write();
        DataRegister::write(value);
    }

    /// # Disable First Port
    /// Attempt to disable the first PS2 port.
    pub fn diable_first_port() {
        unsafe { Self::write(Self::PS2_COMMAND_BYTE_DISABLE_FIRST) };
    }

    /// # Eanble First Port
    /// Attempt to enable the first PS2 port.
    pub fn enable_first_port() {
        unsafe { Self::write(Self::PS2_COMMAND_BYTE_ENABLE_FIRST) };
    }

    /// # Disable Second Port
    /// Attempt to disable the second port.
    pub fn disable_second_port() {
        unsafe { Self::write(Self::PS2_COMMAND_BYTE_DISABLE_SECOND) };
    }

    /// # Enable Second Port
    /// Attempt to enable the second port.
    pub fn enable_second_port() {
        unsafe { Self::write(Self::PS2_COMMAND_BYTE_ENABLE_SECOND) };
    }

    /// # Test Second Port
    /// Asks the second port to preform a self test, the result of the
    /// self test is returned as a TestStatus.
    pub fn test_second_port() -> TestStatus {
        unsafe { Self::write(Self::PS2_COMMAND_BYTE_TEST_SECOND) };
        Self::wait_read();
        let value = DataRegister::read();

        match value {
            0x00 => TestStatus::TestPassed,
            0x01 => TestStatus::ClockStuckLow,
            0x02 => TestStatus::ClockStuckHigh,
            0x03 => TestStatus::DataStuckLow,
            0x04 => TestStatus::DataStuckHigh,

            _ => unreachable!("PS2 Should only return 0..4 when reading back test data, the machine must be in unstable state. Data={value}")
        }
    }

    /// # Test First Port
    /// Asks the first port to preform a self test, the result of the
    /// self test is returned as a TestStatus.
    pub fn test_first_port() -> TestStatus {
        unsafe { Self::write(Self::PS2_COMMAND_BYTE_TEST_FIRST) };
        Self::wait_read();
        let value = DataRegister::read();

        match value {
            0x00 => TestStatus::TestPassed,
            0x01 => TestStatus::ClockStuckLow,
            0x02 => TestStatus::ClockStuckHigh,
            0x03 => TestStatus::DataStuckLow,
            0x04 => TestStatus::DataStuckHigh,

            _ => unreachable!("PS2 Should only return 0..4 when reading back test data, the machine must be in unstable state. Data={value}")
        }
    }

    /// # Test Controller
    /// Ask the controller to preform a self test, and the result of the
    /// self test is returned as a TestStatus.
    pub fn test_controller() -> TestStatus {
        unsafe { Self::write(Self::PS2_COMMAND_BYTE_TEST_FIRST) };
        Self::wait_read();
        let value = DataRegister::read();

        match value {
            0x55 => TestStatus::TestPassed,
            0xFC => TestStatus::TestFailed,

            _ => unreachable!("PS2 Should only return 0..4 when reading back test data, the machine must be in unstable state. Data={value}")
        }
    }

    /// # Read Controller Output Port
    /// Reads from the controller's output port.
    pub fn read_controller_output_port() -> u8 {
        unsafe { Self::write(Self::PS2_COMMAND_BYTE_READ_OUTPUT_PORT) };
        Self::wait_read();
        DataRegister::read()
    }

    /// # Write Controller Output Port
    /// Writes to the controller's output port.
    pub unsafe fn write_controller_output_port(value: u8) {
        Self::write(Self::PS2_COMMAND_BYTE_WRITE_OUTPUT_PORT);
        Self::wait_write();
        DataRegister::write(value);
    }

    /// # Write First Port's output buffer
    /// Write to the first port's output buffer. This will appear as if the
    /// first port sent some data.
    pub unsafe fn write_first_port_output_buffer(value: u8) {
        Self::write(Self::PS2_COMMAND_BYTE_WRITE_FIRST_OUTPUT);
        Self::wait_write();
        DataRegister::write(value);
    }

    /// # Write Second Port's output buffer
    /// Write to the second port's output buffer. This will appear as if the
    /// second port sent some data.
    pub unsafe fn write_second_port_output_buffer(value: u8) {
        Self::write(Self::PS2_COMMAND_BYTE_WRITE_SECOND_OUTPUT);
        Self::wait_write();
        DataRegister::write(value);
    }

    /// # Write Second Port's input buffer
    /// Write to the second port's input buffer. This will send data to the
    /// connected device on the port.
    pub unsafe fn write_second_port_input_buffer(value: u8) {
        Self::write(Self::PS2_COMMAND_BYTE_WRITE_SECOND_INPUT);
        Self::wait_write();
        DataRegister::write(value);
    }

    /// # System Reset
    /// Preform a reset on the computer. This function does not return!
    pub unsafe fn system_reset() -> ! {
        Self::wait_write();
        loop {
            Self::write(0xFE);
        }
    }
}

/// # Controller Configuration
/// The config byte of the controller used for enabling
/// ports and interrupts. Can set and get information
/// about the controller.
pub struct ControllerConfiguration {}

impl ControllerConfiguration {
    // --- Bit flags of Config
    const FIRST_PORT_INTERRUPT: u8 = 0;
    const SECOND_PORT_INTERRUPT: u8 = 1;
    const SYSTEM_FLAG: u8 = 2;
    const SHOULD_BE_ZERO: u8 = 3;
    const FIRST_PORT_CLOCK: u8 = 4;
    const SECOND_PORT_CLOCK: u8 = 5;
    const FIRST_PORT_TRANSLATION: u8 = 6;
    const MUST_BE_ZERO: u8 = 7;

    /// # Read
    /// Read the ControllerConfiguration from the
    /// CommandRegister at memory address 0.
    pub fn read() -> u8 {
        CommandRegister::read_controller_ram(0)
    }

    /// # Write
    /// Write the ControllerConfiguration using the
    /// CommandRegister.
    pub unsafe fn write(value: u8) {
        CommandRegister::write_controller_ram(0, value);
    }

    /// # Is First Port Interrupt Enabled?
    /// Checks if the first port's interrupts are enabled.
    pub fn is_first_port_interrupt_enabled() -> bool {
        Self::read() & Self::FIRST_PORT_INTERRUPT != 0
    }

    /// # Is Second Port Interrupt Enabled?
    /// Checks if the second port's interrupts are enabled.
    pub fn is_second_port_interrupt_enabled() -> bool {
        Self::read() & Self::SECOND_PORT_INTERRUPT != 0
    }

    /// # Set First Port Interrupt
    /// Sets if the first port's interrupts are going to be enabled.
    /// If enabled the first port uses IRQ 1 to send interrupts. This
    /// is useally mapped to interrupt 33.
    pub fn set_first_port_interrupt(enabled: bool) {
        let mut value = Self::read();
        value.set_bit(Self::FIRST_PORT_INTERRUPT, enabled);
        unsafe { Self::write(value) };
    }

    /// # Set Second Port Interrupt
    /// Sets if the second port's interrupts are going to be enabled.
    /// If enabled the second port uses IRQ 12 to send interrupts. This
    /// is useally mapped to interrupt 44.
    pub fn set_second_port_interrupt(enabled: bool) {
        let mut value = Self::read();
        value.set_bit(Self::SECOND_PORT_INTERRUPT, enabled);
        unsafe { Self::write(value) };
    }

    /// # Assert System Flag
    /// Asserts that the system has POST-ed and ensures the bits are
    /// set. It should be not possible to not pass this, because
    /// how did we boot if the system did not post?
    pub fn assert_system_flag() {
        assert_ne!(
            Self::read() & Self::SYSTEM_FLAG,
            0,
            "The system could not have passed POST, how are we alive?"
        );
    }

    /// # Is First Port Clock Enabled?
    /// Checks if the first port's clock is enabled.
    pub fn is_first_port_clock_enabled() -> bool {
        Self::read() & Self::FIRST_PORT_CLOCK != 0
    }

    /// # Is Second Port Clock Enabled?
    /// Checks if the second port's clock is enabled.
    pub fn is_second_port_clock_enabled() -> bool {
        Self::read() & Self::SECOND_PORT_CLOCK != 0
    }

    /// # Set First Port Clock
    /// Sets if the first port's clock is enabled or disabled.
    pub fn set_first_port_clock(enabled: bool) {
        let mut value = Self::read();
        value.set_bit(Self::FIRST_PORT_CLOCK, enabled);
        unsafe { Self::write(value) };
    }

    /// # Set Second Port Clock
    /// Sets if the second port's clock is enabled or disabled. The
    /// second port is important because it can be used to detect if
    /// the ps2 controller only has one port.
    pub fn set_second_port_clock(enabled: bool) {
        let mut value = Self::read();
        value.set_bit(Self::SECOND_PORT_CLOCK, enabled);
        unsafe { Self::write(value) };
    }

    /// # Is First Port Translation
    /// Checks if the first port is using translation.
    pub fn is_first_port_translation() -> bool {
        Self::read() & Self::FIRST_PORT_TRANSLATION != 0
    }

    /// # Set First Port Translation
    /// Sets if the first port is using translation.
    pub fn set_first_port_translation(enabled: bool) {
        let mut value = Self::read();
        value.set_bit(Self::FIRST_PORT_TRANSLATION, enabled);
        unsafe { Self::write(value) };
    }
}
