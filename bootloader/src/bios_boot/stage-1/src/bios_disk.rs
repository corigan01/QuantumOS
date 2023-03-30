/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
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

use crate::bios_ints::BiosInt;
use crate::error::BootloaderError;
use crate::filesystem::DiskMedia;
use core::ops::Range;

#[repr(packed, C)]
struct DiskAccessPacket {
    packet_size: u8,
    zero: u8,
    number_of_sectors: u16,
    offset: u16,
    segment: u16,
    starting_lba: u64,
}

impl DiskAccessPacket {
    unsafe fn read(&mut self, disk_id: u8) -> Result<(), BootloaderError> {
        let status = BiosInt::read_disk_with_packet(disk_id, self as *mut Self as *mut u8)
            .execute_interrupt();

        if status.did_succeed() {
            return Ok(());
        }

        Err(BootloaderError::DiskIOError)
    }

    unsafe fn write(&mut self, disk_id: u8) -> Result<(), BootloaderError> {
        let status = BiosInt::write_disk_with_packet(disk_id, self as *mut Self as *mut u8)
            .execute_interrupt();

        if status.did_succeed() {
            return Ok(());
        }

        Err(BootloaderError::DiskIOError)
    }
}

#[derive(Clone)]
pub struct BiosDisk {
    drive_id: u8,
}

impl BiosDisk {
    pub fn new(disk_id: u8) -> Self {
        Self { drive_id: disk_id }
    }

    fn construct_packet(
        &self,
        target: *mut u8,
        sector_start: usize,
        sectors: usize,
    ) -> DiskAccessPacket {
        DiskAccessPacket {
            packet_size: 0x10,
            zero: 0,
            number_of_sectors: sectors as u16,
            offset: (target as u64 % 0x10) as u16,
            segment: (target as u64 / 0x10) as u16,
            starting_lba: sector_start as u64,
        }
    }
}

impl DiskMedia for BiosDisk {
    fn read(&self, sector: usize) -> Result<[u8; 512], BootloaderError> {
        let mut tmp = [0u8; 512];

        unsafe {
            self.construct_packet(tmp.as_mut_ptr(), sector, 1)
                .read(self.drive_id)
        }?;

        Ok(tmp)
    }

    unsafe fn read_ptr(&self, sector: usize, ptr: *mut u8) -> Result<(), BootloaderError> {
        self.construct_packet(ptr, sector, 1).read(self.drive_id)
    }
}
