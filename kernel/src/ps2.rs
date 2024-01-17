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
use quantum_lib::x86_64::{io::port::IOPort, tables::idt::InterruptFrame};

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
}
