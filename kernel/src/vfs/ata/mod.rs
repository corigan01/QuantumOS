/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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

use crate::vfs::ata::identify_parser::IdentifyParser;
use crate::vfs::ata::registers::{
    CommandRegister, Commands, DataRegister, DiskID, DriveHeadRegister, ErrorRegister,
    FeaturesRegister, SectorRegisters, StatusFlags, StatusRegister,
};
use crate::vfs::io::{DiskBus, DiskType, ErrorKind, IOError, IOResult, SeekFrom};
use crate::vfs::{io, VFSDisk, VFSDiskID};
use core::error::Error;
use core::fmt::{Display, Formatter};
use owo_colors::OwoColorize;
use qk_alloc::boxed::Box;
use qk_alloc::string::String;
use qk_alloc::vec::Vec;
use quantum_lib::bitset::BitSet;
use quantum_lib::{debug_print, debug_println};
use quantum_utils::human_bytes::HumanBytes;

mod identify_parser;
mod registers;

#[derive(Clone, Copy, Debug)]
pub enum DiskErr {
    FailedToRead,
    MalformedInput,
    Interrupted,
    NotSeekable,
}

impl From<DiskErr> for Box<dyn IOError> {
    fn from(value: DiskErr) -> Self {
        Box::new(value)
    }
}

impl Error for DiskErr {}

impl Display for DiskErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let str = match self {
            DiskErr::FailedToRead => "Failed to read!",
            DiskErr::MalformedInput => "Malformed Input to disk",
            DiskErr::Interrupted => "Disk Operation was Interrupted",
            DiskErr::NotSeekable => "Not a valid Seekable Operation",
        };

        f.write_str(str)?;

        Ok(())
    }
}

impl IOError for DiskErr {
    fn error_kind(&self) -> ErrorKind {
        match self {
            DiskErr::Interrupted => ErrorKind::Interrupted,
            DiskErr::NotSeekable => ErrorKind::NotSeekable,
            _ => ErrorKind::Unknown,
        }
    }
}

pub struct DiskDataCollection {
    raw_sectors_written: usize,
    raw_sectors_read: usize,
    gross_bytes_written: usize,
    gross_bytes_read: usize,
}

impl DiskDataCollection {
    pub fn new() -> Self {
        Self {
            raw_sectors_written: 0,
            raw_sectors_read: 0,
            gross_bytes_written: 0,
            gross_bytes_read: 0,
        }
    }
}

pub struct ATADisk {
    device: DiskID,
    identify: IdentifyParser,
    seek: u64,
    logs: DiskDataCollection
}

impl ATADisk {
    const WORDS_PER_SECTOR: usize = 256;
    const BYTES_PER_SECTOR: usize = Self::WORDS_PER_SECTOR * 2;

    fn new(device: DiskID, identify: Vec<u16>) -> Self {
        Self {
            device,
            identify: IdentifyParser::new(identify),
            seek: 0,
            logs: DiskDataCollection::new(),
        }
    }

    fn io_wait(&self) {
        while StatusRegister::is_status(self.device, StatusFlags::SpinDown)
            && StatusRegister::is_busy(self.device)
            && !StatusRegister::is_err_or_fault(self.device)
            && !StatusRegister::is_status(self.device, StatusFlags::DRQ)
        {}
    }

    fn smart_wait(&self) -> IOResult<()> {
        StatusRegister::perform_400ns_delay(self.device);
        self.io_wait();

        if StatusRegister::is_err_or_fault(self.device) {
            return Err(Box::new(DiskErr::FailedToRead));
        }

        Ok(())
    }

    fn select_sector(&self, sector: usize, sector_count: usize) -> IOResult<()> {
        if sector_count == 0 {
            return Err(Box::new(DiskErr::MalformedInput));
        }

        DriveHeadRegister::select_drive(self.device);
        self.smart_wait()?;

        let small_bits = sector.get_bits(24..28) as u8;
        DriveHeadRegister::set_bits_24_to_27_of_lba(self.device, small_bits);
        FeaturesRegister::reset_to_zero(self.device);
        SectorRegisters::select_sectors(self.device, sector_count as u8);
        SectorRegisters::select_lba_0_to_24_bits(self.device, sector);

        Ok(())
    }

    pub fn read_raw(&mut self, sector: usize, sector_count: usize) -> IOResult<Vec<u8>> {
        self.select_sector(sector, sector_count)?;

        CommandRegister::send_command(self.device, Commands::ReadSectorsPIO);
        self.smart_wait()?;

        let mut new_vec: Vec<u8> = Vec::new();
        for _ in 0..sector_count {
            for _ in 0..Self::WORDS_PER_SECTOR {
                let value = DataRegister::read_u16(self.device);
                let arr: [u8; 2] = value.to_le_bytes();

                for b in arr {
                    new_vec.push(b);
                }
            }

            self.smart_wait()?;
        }

        self.logs.raw_sectors_read += sector_count;

        Ok(new_vec)
    }

    pub fn write_raw(&mut self, buf: &[u8], sector: usize, sector_count: usize) -> IOResult<()> {
        if buf.len() < sector_count * Self::WORDS_PER_SECTOR {
            return Err(Box::new(DiskErr::MalformedInput));
        }

        self.select_sector(sector, sector_count)?;

        CommandRegister::send_command(self.device, Commands::WriteSectorsPIO);
        self.smart_wait()?;

        for sec in 0..sector_count {
            for word in 0..Self::WORDS_PER_SECTOR {
                let idx = ((sec * Self::WORDS_PER_SECTOR) + word) * 2;

                let write_value = ((buf[idx + 1] as u16) << 8) + (buf[idx + 0] as u16);
                DataRegister::write_u16(self.device, write_value);
            }
            self.smart_wait()?;
        }

        CommandRegister::send_command(self.device, Commands::CacheFlush);
        self.smart_wait()?;

        self.logs.raw_sectors_written += sector_count;

        Ok(())
    }

    pub fn total_disk_bytes(&self) -> HumanBytes {
        HumanBytes::from(self.identify.user_sectors_28bit_lba() * Self::BYTES_PER_SECTOR)
    }
}

impl Display for ATADisk {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "{} {:?} {:?} {:?} {} {}",
            self.identify.model_number().as_str(),
            self.identify.interconnect(),
            self.identify.specific_config(),
            self.identify.identify_completion_status(),
            self.identify.max_sectors_per_request(),
            HumanBytes::from(self.identify.user_sectors_28bit_lba() * 512)
        )?;

        Ok(())
    }
}

fn scan_for_disks() -> Vec<ATADisk> {
    let mut disks: Vec<ATADisk> = Vec::new();

    for disk in DiskID::iter() {
        let disk = *disk;
        debug_print!("Scanning disk '{:?}' \t ", disk);

        // Select the drive that we are using. Since we are preforming a disk change,
        // we must wait >400ns for the controller to push its status on the IO lines.
        DriveHeadRegister::select_drive(disk);
        StatusRegister::perform_400ns_delay(disk);

        // Spec suggests we need to zero all the sector registers before sending the identify command
        SectorRegisters::zero_registers(disk);

        CommandRegister::send_command(disk, Commands::Identify);
        StatusRegister::perform_400ns_delay(disk);

        // If the bus is floating, we know we don't have a disk
        if StatusRegister::is_floating(disk) {
            debug_println!("{}", "N/A".yellow());
            continue;
        }

        // If some bit got set for the sector registers, its not a ATA device.
        // Some ATAPI drives to not follow spec! At this point we *must* stop pulling.
        if !SectorRegisters::are_all_zero(disk) {
            debug_println!("{}", "Skip".yellow());
            continue;
        }

        // Loop while busy
        while StatusRegister::is_busy(disk)
            && !StatusRegister::is_err_or_fault(disk)
            && !StatusRegister::is_status(disk, StatusFlags::DRQ)
        {}

        if StatusRegister::is_err_or_fault(disk) {
            let errors = ErrorRegister::all_flags(disk);

            debug_println!("{}\nError Details: {:#?}\n", "ERR".red().bold(), errors);
            continue;
        }

        if !StatusRegister::is_status(disk, StatusFlags::DRQ) {
            unreachable!("The 'DRQ' should be set at this point");
        }

        // Finally: Read the Identify Response
        let mut read_identify = Vec::new();
        for _ in 0..256 {
            let value = DataRegister::read_u16(disk);
            read_identify.push(value);
        }

        let disk = ATADisk::new(disk, read_identify);

        debug_println!(
            "{}\t{}",
            "OK".green().bold(),
            disk.identify.model_number().as_str().dimmed()
        );

        disks.push(disk);
    }

    disks
}

pub fn init_ata_disks() {
    let mut disks = scan_for_disks();
    while let Some(disk) = disks.pop_front() {
        let boxed_disk: Box<dyn VFSDisk> = Box::new(disk);

        VFSDiskID::publish_disk(boxed_disk);
    }
}

impl io::Seek for ATADisk {
    fn seek(&mut self, seek: SeekFrom) -> IOResult<u64> {
        let total_bytes = self.total_disk_bytes().into();
        self.seek = seek
            .modify_pos(total_bytes, 0, self.seek)
            .ok_or(DiskErr::NotSeekable)?;

        Ok(self.seek)
    }
}

impl io::Read for ATADisk {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize> {
        let amount_to_read = buf.len();
        let current_idx = self.seek as usize;

        let translated_sector = current_idx / Self::BYTES_PER_SECTOR;
        let sect_across_bounds = current_idx % Self::BYTES_PER_SECTOR;
        let amount_of_sectors =
            ((amount_to_read + sect_across_bounds) / Self::BYTES_PER_SECTOR) + 1;

        // TODO: since we are already reading into a buf, why do we have to read into a vector?
        // FIXME: If we read too many sectors with `read_raw`, it will not complete successfully,
        //        so we need some way in the future to call read_raw multiple times if needed!
        let read = self.read_raw(translated_sector, amount_of_sectors)?;

        for i in 0..amount_to_read {
            buf[i] = read[i + sect_across_bounds];
        }

        self.logs.gross_bytes_read += amount_to_read;

        Ok(amount_to_read)
    }
}

impl io::Write for ATADisk {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        let amount_to_write = buf.len();
        let current_idx = self.seek as usize;

        let translated_sector = current_idx / Self::BYTES_PER_SECTOR;
        let sect_across_bounds = current_idx % Self::BYTES_PER_SECTOR;
        let amount_of_sectors =
            ((amount_to_write + sect_across_bounds) / Self::BYTES_PER_SECTOR) + 1;

        // FIXME: When writing small amounts in-between sector boundaries, we should not have
        //        to read the data, *then* write it back!
        let mut read = self.read_raw(translated_sector, amount_of_sectors)?;

        for i in 0..amount_to_write {
            read[i + sect_across_bounds] = buf[i];
        }

        self.write_raw(read.as_slice(), translated_sector, amount_of_sectors)?;

        self.logs.gross_bytes_written += amount_to_write;

        Ok(amount_to_write)
    }

    fn flush(&mut self) -> IOResult<()> {
        // TODO: We should buffer the write operation, and actually take the flush into account!
        Ok(())
    }
}

impl io::DiskInfo for ATADisk {
    fn disk_type(&self) -> DiskType {
        DiskType::HardDisk
    }

    fn disk_bus(&self) -> DiskBus {
        DiskBus::ParallelPIO
    }

    fn disk_model(&self) -> String {
        self.identify.model_number()
    }

    fn disk_capacity(&self) -> HumanBytes {
        self.total_disk_bytes()
    }
}
