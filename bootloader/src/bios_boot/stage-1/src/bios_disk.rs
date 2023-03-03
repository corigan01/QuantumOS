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


use core::ops::{Range};
use crate::bios_ints::BiosInt;


#[repr(packed(2), C)]
#[derive(Debug)]
struct DiskAccessPacket {
    packet_size: u8,
    zero: u8,
    number_of_sectors: u16,
    offset: u16,
    segment: u16,
    starting_lba: u64,
}

impl DiskAccessPacket {
    fn new_zero() -> Self {
        Self {
            packet_size: 0,
            zero: 0,
            number_of_sectors: 0,
            offset: 0,
            segment: 0,
            starting_lba: 0
        }
    }

    unsafe fn read(&mut self, disk_id: u8) {
        BiosInt::read_disk_with_packet(
            disk_id,
            self as *mut Self as *mut u8)
            .execute_interrupt().unwrap();
    }

    unsafe fn write(&mut self, disk_id: u8) {
        BiosInt::write_disk_with_packet(
            disk_id,
            self as *mut Self as *mut u8)
            .execute_interrupt().unwrap();
    }


}

pub struct BiosDisk {
    drive_id: u8
}

impl BiosDisk {
    pub fn new(disk_id: u8) -> Self {
        Self {
            drive_id: disk_id
        }
    }

    fn get_packet(&self, target: *mut u8, sector_start: u16, sectors: u16) -> DiskAccessPacket {
        DiskAccessPacket {
            packet_size: 0x10,
            number_of_sectors: sectors,
            offset: target as u16 % 0x10,
            segment: target as u16 / 0x10,
            starting_lba: sector_start as u64,
            ..DiskAccessPacket::new_zero()
        }
    }

    pub unsafe fn read_from_disk(&self, target: *mut u8, sectors: Range<u16>) {
        self.get_packet(target, sectors.start, sectors.count() as u16)
            .read(self.drive_id);
    }

    pub unsafe fn write_to_disk(&self, target: *mut u8, sectors: Range<u16>) {
        self.get_packet(target, sectors.start, sectors.count() as u16)
            .write(self.drive_id);
    }

}