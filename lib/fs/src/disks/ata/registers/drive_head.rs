/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
  Part of the Quantum OS Project

Copyright 2023 Gavin Kellam

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

use super::{DiskID, ResolveIOPortBusOffset, DRIVE_HEAD_OFFSET_FROM_IO_BASE};

static mut LAST_DISK: Option<DiskID> = None;

pub struct DriveHeadRegister {}
impl ResolveIOPortBusOffset<DRIVE_HEAD_OFFSET_FROM_IO_BASE> for DriveHeadRegister {}

impl DriveHeadRegister {
    const ATA_DRV: u8 = 4;
    const ATA_LBA: u8 = 6;

    pub unsafe fn read(device: DiskID) -> u8 {
        unsafe { Self::bus_io(device).read_u8() }
    }

    pub unsafe fn write(device: DiskID, value: u8) {
        unsafe { Self::bus_io(device).write_u8(value) }
    }

    pub fn is_using_chs(device: DiskID) -> bool {
        (unsafe { Self::read(device) } & (1 << Self::ATA_LBA)) == 0
    }

    pub fn lba_bits_24_27(device: DiskID, lba_bits: u8) {
        assert!(
            lba_bits & 0b11111000 > 0,
            "Should not be sending more then 3 bits to DriveHeadRegister"
        );

        let read_reg = unsafe { Self::read(device) };
        unsafe { Self::write(device, (read_reg & !0b111) | lba_bits) }
    }

    pub fn is_using_lba(device: DiskID) -> bool {
        !Self::is_using_chs(device)
    }

    pub fn clear_select_cache() {
        unsafe { LAST_DISK = None }
    }
}
