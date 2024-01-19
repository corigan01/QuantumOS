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

use self::registers::{CommandRegister, DataRegister, ReadRegister, StatusRegister, WriteRegister};
use crate::{
    pic::pic_eoi,
    ps2::registers::{ControllerConfiguration, TestStatus},
};
use owo_colors::OwoColorize;
use qk_alloc::vec::Vec;
use quantum_lib::{debug_print, debug_println, x86_64::tables::idt::InterruptFrame};

mod registers;

// FIXME: There are more types of Devices that we should support. For now, we only support
//        basic mouse and keyboard from QEMU.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DeviceType {
    Keyboard,
    Mouse,
}

impl TryFrom<&[u8]> for DeviceType {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        assert!(
            value.len() <= 2 && value.len() > 0,
            "Input bytes must be larger then 0, but smaller or equal to 2. Got len={} instead.",
            value.len()
        );

        match value {
            &[0x00] | &[0x03] | &[0x04] => Ok(DeviceType::Mouse),
            &[0xAB, 0x84] | &[0xAB, 0x85] | &[0xAB, 0x86] => Ok(DeviceType::Keyboard),
            _ => Err("Could not detect type from input sequence."),
        }
    }
}

static mut FIRST_PORT_RECV_BUFFER: Vec<u8> = Vec::new();
static mut SECOND_PORT_RECV_BUFFER: Vec<u8> = Vec::new();

pub fn interrupt_handler_first_port(
    _frame: InterruptFrame,
    _interrupt_id: u8,
    _error_code: Option<u64>,
) {
    unsafe {
        FIRST_PORT_RECV_BUFFER.push(DataRegister::read());
    }
    unsafe { pic_eoi(1) };
}

pub fn interrupt_handler_second_port(
    _frame: InterruptFrame,
    _interrupt_id: u8,
    _error_code: Option<u64>,
) {
    unsafe {
        SECOND_PORT_RECV_BUFFER.push(DataRegister::read());
    }
    unsafe { pic_eoi(12) };
}

fn wait_read() {
    loop {
        let status = StatusRegister::get_status();

        if status.check_flag(registers::StatusFlags::InputBufferEmpty)
            || status.check_flag(registers::StatusFlags::TimeoutError)
        {
            break;
        }
    }
}

unsafe fn write_first_ps2_port(value: u8) {
    wait_read();
    DataRegister::write(value);
}

unsafe fn write_second_ps2_port(value: u8) {
    CommandRegister::write_second_port_input_buffer(value);
}

fn clear_recv_buffers() {
    unsafe {
        FIRST_PORT_RECV_BUFFER.remove_all();
        SECOND_PORT_RECV_BUFFER.remove_all();
    }
}

const DISABLE_SCANNING_COMMAND: u8 = 0xF5;
const ENABLE_SCANNING_COMMAND: u8 = 0xF4;
const IDENTIFY_COMMAND: u8 = 0xF2;

const DEVICE_RESP_ACK: u8 = 0xFA;

pub fn ps2_init() -> Result<(), &'static str> {
    debug_print!("Init PS/2 Controller ");

    CommandRegister::diable_first_port();
    CommandRegister::disable_second_port();
    ControllerConfiguration::set_first_port_translation(false);

    for _ in 0..16 {
        DataRegister::read();
    }

    if CommandRegister::test_controller() != TestStatus::TestPassed {
        debug_println!("Controller self test failed!");
        return Err("Controller self test failed");
    }

    debug_println!("{}", "OK".green().bold());

    ControllerConfiguration::set_first_port_interrupt(false);
    ControllerConfiguration::set_second_port_clock(false);
    let is_second_port = if ControllerConfiguration::is_second_port_clock_enabled() {
        debug_println!("Found PS/2 controller with one port");
        false
    } else {
        debug_println!("Found PS/2 controller with two ports");
        ControllerConfiguration::set_second_port_clock(true);
        ControllerConfiguration::set_second_port_interrupt(false);
        true
    };

    let is_first = if CommandRegister::test_first_port() == TestStatus::TestPassed {
        debug_println!("Found PS/2 Device on port 0");
        ControllerConfiguration::set_first_port_clock(true);
        ControllerConfiguration::set_first_port_interrupt(true);
        true
    } else {
        false
    };

    let is_second =
        if is_second_port && CommandRegister::test_second_port() == TestStatus::TestPassed {
            debug_println!("Found PS/2 Device on port 1");
            ControllerConfiguration::set_second_port_clock(true);
            ControllerConfiguration::set_second_port_interrupt(true);
            true
        } else {
            false
        };

    if is_first {
        CommandRegister::enable_first_port();
    }

    if is_second {
        CommandRegister::enable_second_port();
    }

    clear_recv_buffers();

    if is_first || is_second {
        Ok(())
    } else {
        debug_println!("No PS/2 devices found");
        Err("No devices detected")
    }
}
