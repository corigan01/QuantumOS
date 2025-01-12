/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

use crate::{interrupts::assert_interrupts, io::IOPort};

const CHANNEL_0_DATA: IOPort = IOPort::new(0x40);
const _CHANNEL_1_DATA: IOPort = IOPort::new(0x41);
const _CHANNEL_2_DATA: IOPort = IOPort::new(0x42);
const COMMAND: IOPort = IOPort::new(0x43);

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum PitSelectChannel {
    Channel0 = 0,
    Channel1 = 1,
    Channel2 = 2,
    ReadBack = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum PitAccessMode {
    LatchCount = 0,
    AccessLoOnly = 1,
    AccessHiOnly = 2,
    AccessLoHi = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum PitOperatingMode {
    TerminalCount = 0,
    RetriggerableOneShot = 1,
    RateGenerator = 2,
    SquareWave = 3,
    SoftwareStrobe = 4,
    HardwareStrobe = 5,
}

pub fn pit_command(
    channel: PitSelectChannel,
    access: PitAccessMode,
    mode: PitOperatingMode,
    bcd_mode: bool,
) {
    let bcd_bit = if bcd_mode { 1 } else { 0 };
    let mode_bit = (mode as u8) << 1;
    let access_bit = (access as u8) << 4;
    let channel_bit = (channel as u8) << 6;

    unsafe {
        COMMAND.write_byte(bcd_bit | mode_bit | access_bit | channel_bit);
    }
}

/// Set the pit reload count.
///
/// # Interrupts
/// Interrupts must be disabled before calling this function!
pub fn set_pit_reload(count: u16) {
    assert_interrupts(false);

    unsafe {
        CHANNEL_0_DATA.write_byte((count & 0xFF) as u8);
        CHANNEL_0_DATA.write_byte(((count >> 8) & 0xFF) as u8);
    }
}

/// Set the pit reload count in HZ.
///
/// # Interrupts
/// Interrupts must be disabled before calling this function!
pub fn set_pit_hz(hz: f32) -> f32 {
    assert_interrupts(false);

    let div = 1193182_f32 / hz;
    let int_div = div as u16;

    set_pit_reload(int_div);

    1193182_f32 / (int_div as f32)
}
