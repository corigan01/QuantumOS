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

/// # Ports
/// COMMS ports for serial io on x86.
pub(crate) mod ports {
    use arch::io::IOPort;

    pub const COM1: IOPort = IOPort::new(0x3f8);
    pub const COM2: IOPort = IOPort::new(0x2f8);
    pub const COM3: IOPort = IOPort::new(0x3E8);
    pub const COM4: IOPort = IOPort::new(0x2e8);
    pub const COM5: IOPort = IOPort::new(0x5f8);
    pub const COM6: IOPort = IOPort::new(0x4f8);
    pub const COM7: IOPort = IOPort::new(0x5e8);
    pub const COM8: IOPort = IOPort::new(0x4e8);

    pub const COMMS_ARRAY: [IOPort; 8] = [COM1, COM2, COM3, COM4, COM5, COM6, COM7, COM8];
}

#[allow(unused)]
mod offsets {
    /// # (Read) Receive Buffer Register Offset
    pub const R_RECEIVE_BUFFER: u16 = 0;

    /// # (Write) Transmit Buffer Register Offset
    pub const W_TRANSMIT_BUFFER: u16 = 0;

    /// # (Read/Write) Interrupt Enable Register Offset
    pub const RW_INTERRUPT_ENABLE: u16 = 1;

    /// # (Read/Write) DLAB Least Significant Byte Register Offset
    /// -- DLAB must be set to '1'
    pub const RW_DLAB_LSB: u16 = 0;

    /// # (Read/Write) DLAB Most Sigificant Byte Register Offset
    /// -- DLAB must be set to '1'
    pub const RW_DLAB_MSB: u16 = 1;

    /// # (Read) Interrupt Identification Register Offset
    pub const R_INTERRUPT_IDENTIFICATION: u16 = 2;

    /// # (Write) FIFO Control Register Offset
    pub const W_FIFO_CONTROL: u16 = 2;

    /// # (Read/Write) Line Control Register Offset
    /// -- Most significant bit is the DLAB setting
    pub const RW_LINE_CONTROL: u16 = 3;

    /// # (Read/Write) Modem Control Register Offset
    pub const RW_MODEM_CONTROL: u16 = 4;

    /// # (Read) Line Status Register Offset
    pub const R_LINE_STATUS: u16 = 6;

    /// # (Read/Write) Scratch Register Offset
    pub const RW_SCRATCH: u16 = 7;
}

macro_rules! impl_reg {
    (R: $func_name: ident, $port_offset: expr) => {
        #[allow(unused)]
        #[inline(always)]
        pub unsafe fn $func_name(port: arch::io::IOPort) -> u8 {
            (port + $port_offset).read_byte()
        }
    };

    (W: $func_name: ident, $port_offset: expr) => {
        #[allow(unused)]
        #[inline(always)]
        pub unsafe fn $func_name(port: arch::io::IOPort, value: u8) {
            (port + $port_offset).write_byte(value)
        }
    };

    (RW: $read_name: ident, $write_name: ident, $port_offset: expr) => {
        impl_reg!(R: $read_name, $port_offset);
        impl_reg!(W: $write_name, $port_offset);
    };
}

impl_reg!(R: read_receive_buffer, offsets::R_RECEIVE_BUFFER);
impl_reg!(W: write_transmit_buffer, offsets::W_TRANSMIT_BUFFER);
impl_reg!(RW: read_interrupt_enable, write_interrupt_enable, offsets::RW_INTERRUPT_ENABLE);
impl_reg!(RW: read_dlab_lsb, write_dlab_lsb, offsets::RW_DLAB_LSB);
impl_reg!(RW: read_dlab_msb, write_dlab_msb, offsets::RW_DLAB_MSB);
impl_reg!(R: read_interrupt_identification, offsets::R_INTERRUPT_IDENTIFICATION);
impl_reg!(W: write_fifo_control, offsets::W_FIFO_CONTROL);
impl_reg!(RW: read_line_control, write_line_control, offsets::RW_LINE_CONTROL);
impl_reg!(RW: read_modem_control, write_modem_control, offsets::RW_MODEM_CONTROL);
impl_reg!(R: read_line_status, offsets::R_LINE_STATUS);
impl_reg!(RW: read_scratch, write_scratch, offsets::RW_SCRATCH);

// FIXME: I am not sure how I want to impl this, I just want to get some
//        debug info right now and want to get a basic product out. I
//        should rewrite this when I get back into the kernel :^)
