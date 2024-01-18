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

use crate::pic::pic_eoi;
use quantum_lib::{
    bitset::BitSet,
    x86_64::{io::port::IOPort, tables::idt::InterruptFrame},
};

const PS2_CONTROLLER_DATA_PORT: IOPort = IOPort::new(0x60);
const PS2_CONTROLLER_STATUS_PORT: IOPort = IOPort::new(0x64);
const PS2_CONTROLLER_COMMAND_PORT: IOPort = IOPort::new(0x64);

// FIXME: We should probebly do something where we have a proc_macro or something,
//        handle this? It should be not up to the kernel programmer to remember
//        that this is here and needs to be attached.
pub fn ps2_interrupt_attachment(_frame: InterruptFrame, interrupt_id: u8, _error: Option<u64>) {
    unsafe { pic_eoi(interrupt_id) }
}

pub struct StatusFlags {
    pub output_buffer_status: bool,
    pub input_buffer_status: bool,
    pub system_flag: bool,
    pub command_or_data: bool,
    pub timeout_error: bool,
    pub parity_error: bool,
}

impl StatusFlags {
    pub fn read() -> Self {
        let port = PS2_CONTROLLER_STATUS_PORT;
        let value = unsafe { port.read_u8() };

        StatusFlags {
            output_buffer_status: value & (1 << 0) != 0,
            input_buffer_status: value & (1 << 1) != 0,
            system_flag: value & (1 << 2) != 0,
            command_or_data: value & (1 << 3) != 0,
            timeout_error: value & (1 << 6) != 0,
            parity_error: value & (1 << 7) != 0,
        }
    }

    pub fn wait_ready() {
        loop {
            let flags = Self::read();

            if !flags.output_buffer_status {
                break;
            }
        }
    }

    pub fn wait_data() {
        loop {
            let flags = Self::read();

            if flags.output_buffer_status {
                break;
            }
        }
    }
}

pub enum TestStatus {
    Passed,
    Failed,
    ClockStuckHigh,
    ClockStuckLow,
    DataStuckLow,
    DataStuckHigh,
}

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

unsafe fn ps2_read_ram(address: u8) -> u8 {
    assert!(
        address <= PS2_COMMAND_BYTE_WRITE_END - PS2_COMMAND_BYTE_WRITE_BEGIN,
        "Address goes out of range of ps2"
    );
    let command_byte = PS2_COMMAND_BYTE_READ_BEGIN + address;
    PS2_CONTROLLER_COMMAND_PORT.write_u8(command_byte);
    StatusFlags::wait_data();
    PS2_CONTROLLER_DATA_PORT.read_u8()
}

unsafe fn ps2_write_ram(address: u8, value: u8) {
    assert!(
        address <= PS2_COMMAND_BYTE_WRITE_END - PS2_COMMAND_BYTE_WRITE_BEGIN,
        "Address goes out of range of ps2"
    );
    let command_byte = PS2_COMMAND_BYTE_WRITE_BEGIN + address;
    PS2_CONTROLLER_COMMAND_PORT.write_u8(command_byte);
    StatusFlags::wait_ready();
    PS2_CONTROLLER_DATA_PORT.write_u8(value);
}

unsafe fn ps2_disable_first_port() {
    PS2_CONTROLLER_COMMAND_PORT.write_u8(PS2_COMMAND_BYTE_DISABLE_FIRST);
}

unsafe fn ps2_enable_first_port() {
    PS2_CONTROLLER_COMMAND_PORT.write_u8(PS2_COMMAND_BYTE_ENABLE_FIRST);
}

unsafe fn ps2_disable_second_port() {
    PS2_CONTROLLER_COMMAND_PORT.write_u8(PS2_COMMAND_BYTE_DISABLE_SECOND);
}

unsafe fn ps2_enable_second_port() {
    PS2_CONTROLLER_COMMAND_PORT.write_u8(PS2_COMMAND_BYTE_ENABLE_SECOND);
}

unsafe fn ps2_test_controller() -> TestStatus {
    PS2_CONTROLLER_COMMAND_PORT.write_u8(PS2_COMMAND_BYTE_TEST_CONTROLLER);
    StatusFlags::wait_data();
    let status = PS2_CONTROLLER_DATA_PORT.read_u8();

    match status {
        0x55 => TestStatus::Passed,
        0xFC => TestStatus::Failed,

        _ => panic!("PS2 Controller should be recv this byte when testing ... byte={status}"),
    }
}

unsafe fn ps2_test_second_port() -> TestStatus {
    PS2_CONTROLLER_COMMAND_PORT.write_u8(PS2_COMMAND_BYTE_TEST_SECOND);
    StatusFlags::wait_data();
    let status = PS2_CONTROLLER_DATA_PORT.read_u8();

    match status {
        0x00 => TestStatus::Passed,
        0x01 => TestStatus::ClockStuckLow,
        0x02 => TestStatus::ClockStuckHigh,
        0x03 => TestStatus::DataStuckLow,
        0x04 => TestStatus::DataStuckHigh,

        _ => panic!("PS2 Controller should be recv this byte when testing ... byte={status}"),
    }
}

unsafe fn ps2_test_first_port() -> TestStatus {
    PS2_CONTROLLER_COMMAND_PORT.write_u8(PS2_COMMAND_BYTE_TEST_FIRST);
    StatusFlags::wait_data();
    let status = PS2_CONTROLLER_DATA_PORT.read_u8();

    match status {
        0x00 => TestStatus::Passed,
        0x01 => TestStatus::ClockStuckLow,
        0x02 => TestStatus::ClockStuckHigh,
        0x03 => TestStatus::DataStuckLow,
        0x04 => TestStatus::DataStuckHigh,

        _ => panic!("PS2 Controller should be recv this byte when testing ... byte={status}"),
    }
}

unsafe fn ps2_read_controller_output_port() -> u8 {
    PS2_CONTROLLER_COMMAND_PORT.write_u8(PS2_COMMAND_BYTE_READ_OUTPUT_PORT);
    StatusFlags::wait_data();
    PS2_CONTROLLER_DATA_PORT.read_u8()
}

unsafe fn ps2_write_controller_output_port(value: u8) {
    PS2_CONTROLLER_COMMAND_PORT.write_u8(PS2_COMMAND_BYTE_WRITE_OUTPUT_PORT);
    StatusFlags::wait_ready();
    PS2_CONTROLLER_DATA_PORT.write_u8(value);
}

unsafe fn ps2_write_first_output_buffer(value: u8) {
    PS2_CONTROLLER_COMMAND_PORT.write_u8(PS2_COMMAND_BYTE_WRITE_FIRST_OUTPUT);
    StatusFlags::wait_ready();
    PS2_CONTROLLER_DATA_PORT.write_u8(value);
}

unsafe fn ps2_write_second_output_buffer(value: u8) {
    PS2_CONTROLLER_COMMAND_PORT.write_u8(PS2_COMMAND_BYTE_WRITE_SECOND_OUTPUT);
    StatusFlags::wait_ready();
    PS2_CONTROLLER_DATA_PORT.write_u8(value);
}

unsafe fn ps2_write_second_input_buffer(value: u8) {
    PS2_CONTROLLER_COMMAND_PORT.write_u8(PS2_COMMAND_BYTE_WRITE_SECOND_INPUT);
    StatusFlags::wait_ready();
    PS2_CONTROLLER_DATA_PORT.write_u8(value);
}

pub struct Ps2Configuration(u8);

impl Ps2Configuration {
    const FIRST_PORT_INTERRUPT_BIT: u8 = 1 << 0;
    const SECOND_PORT_INTERRUPT_BIT: u8 = 1 << 1;
    const SYSTEM_FLAG_BIT: u8 = 1 << 2;
    const FIRST_PORT_CLOCK_BIT: u8 = 1 << 4;
    const SECOND_PORT_CLOCK_BIT: u8 = 1 << 5;
    const FIRST_PORT_TRANSLATION_BIT: u8 = 1 << 6;

    pub fn read() -> Ps2Configuration {
        let ram0 = unsafe { ps2_read_ram(0) };
        assert!(
            ram0 & Self::SYSTEM_FLAG_BIT != 0,
            "Somehow the system booted with the POST flag disabled!"
        );

        Ps2Configuration(ram0)
    }

    pub fn is_first_port_interrupts_enabled(&self) -> bool {
        self.0 & Self::FIRST_PORT_INTERRUPT_BIT != 0
    }

    pub fn is_second_port_interrupts_enabled(&self) -> bool {
        self.0 & Self::SECOND_PORT_INTERRUPT_BIT != 0
    }

    pub fn is_first_port_clock_enabled(&self) -> bool {
        self.0 & Self::FIRST_PORT_CLOCK_BIT != 0
    }

    pub fn is_second_port_clock_enabled(&self) -> bool {
        self.0 & Self::SECOND_PORT_CLOCK_BIT != 0
    }

    pub fn set_first_port_interrupt(&mut self, enabled: bool) {
        self.0.set_bit(0, enabled);
        unsafe { ps2_write_ram(0, self.0) };
    }

    pub fn set_second_port_interrupt(&mut self, enabled: bool) {
        self.0.set_bit(1, enabled);
        unsafe { ps2_write_ram(0, self.0) };
    }
}
