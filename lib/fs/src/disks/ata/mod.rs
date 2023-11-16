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

use qk_alloc::{format, vec, vec::Vec};
use quantum_lib::bitset::BitSet;

use self::registers::{
    command::{CommandRegister, Commands},
    drive_head::DriveHeadRegister,
    sector::SectorRegister,
    status::{StatusFlags, StatusRegister},
    WriteRegisterBus,
};
use super::cache::DiskCache;
pub use crate::disks::ata::registers::DiskID;
use crate::{
    disks::ata::{
        identify::IdentifyParser,
        registers::{
            data::DataRegister, error::ErrorRegister, feature::FeatureRegister, ReadRegisterBus,
        },
    },
    error::{FsError, FsErrorKind},
    io::{Read, Seek, SeekFrom, Write},
    FsResult,
};
use core::{cmp::min, i64, marker::PhantomData, u64};

mod identify;
mod registers;

pub struct UnknownState {}
pub struct Quarried {}

pub struct AtaDisk<Any = UnknownState> {
    disk_id: DiskID,
    cache: DiskCache,
    seek: u64,
    identify: IdentifyParser,
    phan: PhantomData<Any>,
}

impl DiskID {}

impl AtaDisk {
    /// # New
    /// Constructs a new empty disk. At this point the disk will know nothing about itself, and
    /// will need to be quarried. Quarrying the disk changes its state, and more functions will
    /// then become available.
    pub fn new(disk: DiskID) -> Self {
        Self {
            disk_id: disk,
            cache: DiskCache::new(20),
            seek: 0,
            identify: unsafe { IdentifyParser::empty() },
            phan: PhantomData,
        }
    }

    /// # Quarry
    /// Quarries the disk to get basic info, and to also set it up. Quarrying can fail in a number
    /// of ways. The disk could not be found on the bus (FsErrorKind::NotFound). The disk could be
    /// a non ATA type disk (FsErrorKind::Unsupported). The disk could timeout when trying to
    /// quarry (FsErrorKind::TimedOut).
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
            cache: self.cache,
            seek: 0,
            identify: identify_parser,
            phan: PhantomData,
        })
    }
}

impl AtaDisk<Quarried> {
    /// # Wait for Disk
    /// Pulls until the disk is ready for the next command. This is quite slow, and should use DMA
    /// as soon as possible.
    fn wait_for_disk(&self) -> FsResult<()> {
        // Waits 400ns for the IDE bus to propagate changes
        StatusRegister::perform_400ns_delay(self.disk_id);
        loop {
            let status = StatusRegister::get_status(self.disk_id);

            // Check if the disk is in an error state
            if status.check_flag(StatusFlags::ErrorOccurred) {
                return Err(FsError::from_string(
                    FsErrorKind::Other,
                    format!(
                        "Drive Error Occurred: {:#?}",
                        ErrorRegister::get_errors(self.disk_id)
                    ),
                ));
            }

            // Check if there was a fault
            if status.check_flag(StatusFlags::DriveFault) {
                return Err(FsError::new(FsErrorKind::Other, "Drive Fault Occured"));
            }

            // Pull until not busy
            if !status.check_flag(StatusFlags::Busy) || status.check_flag(StatusFlags::Ready) {
                break;
            }
        }

        Ok(())
    }

    /// # Seek Sector
    /// Sets the sector for the disk to read/write.
    fn seek_sector(&self, sector: usize) {
        let small_bits = sector.get_bits(24..28) as u8;
        DriveHeadRegister::lba_bits_24_27(self.disk_id, small_bits);
        FeatureRegister::reset_register(self.disk_id);
        SectorRegister::lba_bits_0_24(self.disk_id, sector & 0xFFFFFF);
    }

    /// # Select Sector Count
    /// Sets the amount of sectors the next operation is going to use.
    fn select_sector_count(&self, count: usize) {
        SectorRegister::set_sectors_per_operation(self.disk_id, count as u8);
    }

    /// # Gets the words per sector on the disk
    pub fn words_per_sector(&self) -> usize {
        self.identify.logical_sector_size() / 2
    }

    /// # Read Raw Sectors
    /// Reads sectors off the disk very rawly.
    pub unsafe fn read_raw_sectors(
        &mut self,
        sector: usize,
        sector_count: usize,
        buffer: &mut [u8],
    ) -> FsResult<usize> {
        let words_per_sector = self.words_per_sector();

        if buffer.len() < sector * (words_per_sector * 2) {
            return Err(FsError::new(
                FsErrorKind::InvalidInput,
                "Cannot write to a buffer with a size that is less then the requested sector size",
            ));
        }

        self.select_sector_count(sector_count);
        self.seek_sector(sector);

        CommandRegister::send_command(self.disk_id, Commands::ReadSectorsPIO);
        self.wait_for_disk()?;

        let mut amount_read = 0;
        let buffer_u16 = buffer.as_ptr() as *mut u16;

        for _ in 0..sector_count {
            for _ in 0..words_per_sector {
                buffer_u16
                    .add(amount_read)
                    .write_unaligned(DataRegister::read_u16(self.disk_id));

                amount_read += 1;
            }

            // Need to wait for the disk to seek to the next sector
            self.wait_for_disk()?;
        }

        Ok(amount_read * 2)
    }

    /// # Write Raw Sectors
    /// Writes sectors to the disk very rawly.
    unsafe fn write_raw_sectors(
        &mut self,
        sector: usize,
        sector_count: usize,
        buffer: &[u8],
    ) -> FsResult<usize> {
        let words_per_sector = self.words_per_sector();

        if buffer.len() < sector * (words_per_sector * 2) {
            return Err(FsError::new(
                FsErrorKind::InvalidInput,
                "Cannot read from a buffer with a size that is less then the requested sector size",
            ));
        }

        self.select_sector_count(sector_count);
        self.seek_sector(sector);

        CommandRegister::send_command(self.disk_id, Commands::WriteSectorsPIO);
        self.wait_for_disk()?;

        let mut amount_written = 0;
        let buffer_u16 = buffer.as_ptr() as *mut u16;

        for _ in 0..sector_count {
            for _ in 0..words_per_sector {
                let value = buffer_u16.add(amount_written).read_unaligned();

                DataRegister::write_u16(self.disk_id, value);
                amount_written += 1;
            }

            // Need to wait for the disk to seek to the next sector
            self.wait_for_disk()?;
        }

        Ok(amount_written * 2)
    }

    /// # Calculate the current sector positon.
    /// Proves the sector pos from the current seek position.
    fn calculate_seek_sector_offset(&self) -> (usize, usize) {
        let bytes_per_sector = self.words_per_sector() * 2;
        let sector_offset = self.seek as usize / bytes_per_sector;
        let within_sector_offset = self.seek as usize % bytes_per_sector;

        (sector_offset, within_sector_offset)
    }
}

impl Seek for AtaDisk<Quarried> {
    fn seek(&mut self, pos: crate::io::SeekFrom) -> FsResult<u64> {
        match pos {
            SeekFrom::Start(start) => self.seek = start as u64,
            SeekFrom::End(end) => {
                let end_of_disk =
                    self.identify.user_sectors_28bit_lba() * self.words_per_sector() * 2;
                let end = (-end) as usize;

                self.seek = (end_of_disk - end) as u64;
            }
            SeekFrom::Current(current) => {
                self.seek = ((self.seek as i64) + current) as u64;
            }
        };
        Ok(self.seek)
    }
}

impl Read for AtaDisk<Quarried> {
    fn read(&mut self, buf: &mut [u8]) -> FsResult<usize> {
        let (sector, sector_offset) = self.calculate_seek_sector_offset();
        let amount_to_read = buf.len();
        let bytes_per_sector = self.words_per_sector() * 2;

        let mut scratchpad_buffer = vec![0_u8; bytes_per_sector];
        let first_sector_values = match self.cache.get_entry(sector) {
            Some(data) => data,
            None => unsafe {
                self.read_raw_sectors(sector, 1, scratchpad_buffer.as_mut())?;
                scratchpad_buffer.as_slice()
            },
        };

        buf[..min(bytes_per_sector - sector_offset, amount_to_read)].copy_from_slice(
            &first_sector_values
                [sector_offset..min(bytes_per_sector, amount_to_read + sector_offset)],
        );

        //if bytes_per_sector > amount_to_read {
        //    return Ok(buf.len());
        //}

        let mut sectors_written = 1;
        buf[bytes_per_sector - sector_offset..]
            .chunks_mut(bytes_per_sector)
            .try_for_each(|buffer_chunk| -> FsResult<()> {
                let maybe_cached = self.cache.get_entry(sector + sectors_written);

                let read_bytes = match maybe_cached {
                    Some(bytes) => bytes,
                    None => {
                        unsafe {
                            self.read_raw_sectors(
                                sector + sectors_written,
                                1,
                                scratchpad_buffer.as_mut(),
                            )
                        }?;

                        scratchpad_buffer.as_slice()
                    }
                };

                buffer_chunk.copy_from_slice(&read_bytes[..buffer_chunk.len()]);
                sectors_written += 1;
                Ok(())
            })?;

        Ok(buf.len())
    }
}

impl Write for AtaDisk<Quarried> {
    fn write(&mut self, buf: &[u8]) -> FsResult<usize> {
        todo!()
    }

    fn flush(&mut self) -> FsResult<()> {
        todo!()
    }
}
