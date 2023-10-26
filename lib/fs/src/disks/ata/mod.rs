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

use qk_alloc::vec::Vec;

use crate::{
    disks::ata::{
        identify::IdentifyParser,
        registers::{data::DataRegister, DiskID, ReadRegisterBus},
    },
    error::{FsError, FsErrorKind},
    FsResult,
};
use core::marker::PhantomData;

use self::registers::{
    command::{CommandRegister, Commands},
    drive_head::DriveHeadRegister,
    sector::SectorRegister,
    status::{StatusFlags, StatusRegister},
};

mod identify;
mod registers;

pub struct UnknownState {}
pub struct Quarried {}

pub struct AtaDisk<Any = UnknownState> {
    disk_id: DiskID,
    seek: u64,
    identify: IdentifyParser,
    phan: PhantomData<Any>,
}

impl DiskID {}

impl AtaDisk {
    pub fn new(disk: DiskID) -> Self {
        Self {
            disk_id: disk,
            seek: 0,
            identify: unsafe { IdentifyParser::empty() },
            phan: PhantomData,
        }
    }

    pub fn quarry(self) -> FsResult<AtaDisk<Quarried>> {
        let disk = self.disk_id;

        // Ensure we are set to the correct disk, this takes some time for the disk to respond, so
        // we need to wait 400ns to ensure the disk has enough time to update the bus.
        DriveHeadRegister::switch_disk(disk);
        StatusRegister::perform_400ns_delay(disk);

        // Zeroing the registers is a 'special value' and preforms some init and reset. Spec
        // recommends this process when quarrying for disks.
        unsafe { SectorRegister::zero_all_registers(disk) };

        // We want the disk to identify itself so we can learn some info about it. This also takes
        // the disk a sec to respond, so we need another delay.
        CommandRegister::send_command(disk, Commands::Identify);
        StatusRegister::perform_400ns_delay(disk);

        // Check if the bus is present by checking if the bus is floating. If its floating we know
        // the bus is not plugged in, so its impossible to have any drives present.
        if StatusRegister::is_bus_floating(disk) {
            return Err(FsError::new(
                FsErrorKind::NotFound,
                "The disk is not present",
            ));
        }

        // Sector Registers should still be all zero (since we just zeroed them), but ATAPI devices
        // are mean and like to change them. The good thing is that we can use this quark to know
        // if we have a ATAPI device and not a ATA device.
        if !SectorRegister::is_registers_zeroed(disk) {
            return Err(FsError::new(
                FsErrorKind::Unsupported,
                "ATAPI device instead of ATA device found on this bus.",
            ));
        }

        // Lets make sure the drive is not busy, we don't want to bother it.
        loop {
            let status = StatusRegister::get_status(disk);

            // We want to make sure the disk is not error
            if status.check_flag(StatusFlags::ErrorOccurred)
                || status.check_flag(StatusFlags::DriveFault)
            {
                return Err(FsError::new(FsErrorKind::BrokenPipe, "Disk Fault/Error"));
            }

            // We want to stop pulling the status if one of the following is true:
            // * The disk is no longer busy
            // * The disk is now ready
            if !status.check_flag(StatusFlags::Busy) || status.check_flag(StatusFlags::Ready) {
                break;
            }
        }

        // Ensure the disk is ready, as it took it long enough. I think I am going to have a panic
        // attack if it takes any longer
        if !StatusRegister::get_status(disk).check_flag(StatusFlags::Ready) {
            return Err(FsError::new(
                FsErrorKind::TimedOut,
                "Disk is taking too long, I am tired of waiting",
            ));
        }

        // Finally Mr. Disk is ready to talk, so lets listen
        let mut raw_identify_data: Vec<u8> = Vec::with_capacity(512);
        for _ in 0..256 {
            let read_value = DataRegister::read_u16(disk);
            raw_identify_data.push((read_value & 0xFF) as u8);
            raw_identify_data.push(((read_value >> 8) & 0xFF) as u8);
        }

        let identify_parser = IdentifyParser::from(raw_identify_data.as_slice());

        Ok(AtaDisk {
            disk_id: disk,
            seek: 0,
            identify: identify_parser,
            phan: PhantomData,
        })
    }
}
