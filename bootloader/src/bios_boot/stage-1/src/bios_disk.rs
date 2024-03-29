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

use crate::filesystem::DiskMedia;
use quantum_lib::x86_64::bios_call::BiosCall;
use bootloader::error::BootloaderError;

#[repr(packed, C)]
#[derive(Copy, Clone)]
struct DiskAccessPacket {
    packet_size: u8,
    zero: u8,
    number_of_sectors: u16,
    offset: u16,
    segment: u16,
    starting_lba: u64,
}

impl DiskAccessPacket {
    fn get_ptr(&mut self) -> *mut u8 {
        self as *mut Self as *mut u8
    }

    unsafe fn read(&mut self, disk_id: u8) -> Result<(), BootloaderError> {
        let status = BiosCall::new()
            .bit16_call()
            .bios_disk_io(disk_id, self.get_ptr(), false);

        if status.did_succeed() {
            return Ok(());
        }

        Err(BootloaderError::DiskIOError)
    }

    #[allow(dead_code)]
    unsafe fn write(&mut self, disk_id: u8) -> Result<(), BootloaderError> {
        let status = BiosCall::new()
            .bit16_call()
            .bios_disk_io(disk_id, self.get_ptr(), true);

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
        target: u32,
        sector_start: usize,
        sectors: usize,
    ) -> DiskAccessPacket {
        DiskAccessPacket {
            packet_size: 12,
            zero: 0,
            number_of_sectors: sectors as u16,
            offset: (target % 0x10) as u16,
            segment: (target / 0x10) as u16,
            starting_lba: sector_start as u64,
        }
    }
}

impl DiskMedia for BiosDisk {
    fn read(&self, sector: usize) -> Result<[u8; 512], BootloaderError> {
        let mut tmp = [0u8; 512];

        unsafe {
            self.construct_packet(tmp.as_mut_ptr() as u32, sector, 1)
                .read(self.drive_id)
        }?;

        Ok(tmp)
    }

    unsafe fn read_ptr(&self, sector: usize, ptr: *mut u8) -> Result<(), BootloaderError> {
        self.construct_packet(ptr as u32, sector, 1)
            .read(self.drive_id)
    }
}
