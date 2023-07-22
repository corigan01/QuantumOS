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

use core::fmt::{Display, Formatter};
use qk_alloc::vec::Vec;
use quantum_lib::{debug_print, debug_println};
use crate::ata::registers::{CommandRegister, Commands, DataRegister, DiskID, DriveHeadRegister, ErrorRegister, FeaturesRegister, SectorRegisters, StatusFlags, StatusRegister};
use owo_colors::OwoColorize;
use quantum_lib::bitset::BitSet;
use quantum_utils::bytes::Bytes;
use crate::ata::identify_parser::IdentifyParser;
use crate::filesystem::impl_disk::{Medium, MediumErr, SeekFrom};

mod registers;
mod identify_parser;

#[derive(Clone, Copy, Debug)]
pub enum DiskErr {
    FailedToRead,
    MalformedInput
}

pub struct ATADisk {
    device: DiskID,
    identify: IdentifyParser,
    cache: Vec<(usize, Vec<u8>)>,
    current: usize
}

impl ATADisk {
    const WORDS_PER_SECTOR: usize = 256;

    fn new(device: DiskID, identify: Vec<u16>) -> Self {
        Self {
            device,
            identify: IdentifyParser::new(identify),
            cache: Vec::new(),
            current: 0,
        }
    }

    fn io_wait(&self) {
        while
             StatusRegister::is_status(self.device, StatusFlags::SpinDown) &&
             StatusRegister::is_busy(self.device) &&
            !StatusRegister::is_err_or_fault(self.device) &&
            !StatusRegister::is_status(self.device, StatusFlags::DRQ) {}
    }

    fn smart_wait(&self) -> Result<(), DiskErr> {
        StatusRegister::perform_400ns_delay(self.device);
        self.io_wait();

        if StatusRegister::is_err_or_fault(self.device) {
            return Err(DiskErr::FailedToRead);
        }

        Ok(())
    }

    fn select_sector(&self, sector: usize, sector_count: usize) -> Result<(), DiskErr> {
        if sector_count == 0 {
            return Err(DiskErr::MalformedInput);
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

    pub fn read_raw(&mut self, sector: usize, sector_count: usize) -> Result<Vec<u8>, DiskErr> {
        self.select_sector(sector, sector_count)?;

        CommandRegister::send_command(self.device, Commands::ReadSectorsPIO);
        self.smart_wait()?;

        let mut new_vec: Vec<u8> = Vec::new();
        for _ in 0..sector_count {
            for _ in 0..Self::WORDS_PER_SECTOR {
                let value = DataRegister::read_u16(self.device);
                let arr = value.to_be_bytes();

                for b in arr {
                    new_vec.push(b);
                }
            }

            self.smart_wait()?;
        }

        Ok(new_vec)
    }

    pub fn write_raw(&mut self, mut data: Vec<u8>, sector: usize, sector_count: usize) -> Result<(), DiskErr> {
        if data.len() < sector_count * Self::WORDS_PER_SECTOR {
            return Err(DiskErr::MalformedInput);
        }

        self.select_sector(sector, sector_count)?;

        CommandRegister::send_command(self.device, Commands::WriteSectorsPIO);
        self.smart_wait()?;

        self.invalidate_cached_sectors(sector, sector_count);

        for _ in 0..sector_count {
            for _ in 0..Self::WORDS_PER_SECTOR {
                let write_value = ((data.remove(1) as u16) << 8) + (data.remove(0) as u16);
                DataRegister::write_u16(self.device, write_value);
            }
            self.smart_wait()?;
        }

        CommandRegister::send_command(self.device, Commands::CacheFlush);
        self.smart_wait()?;

        Ok(())
    }

    pub fn invalidate_cached_sectors(&mut self, base_lba: usize, amount: usize) {
        self.cache.retain(|(sector, _)| {
            *sector < base_lba && *sector > (base_lba + amount)
        });
    }

    pub fn collect_cached_sectors(&self, lba: usize, amount: usize) -> Option<Vec<u8>> {
        // Regardless if we have to loop over the cache multiple times,
        // its way faster to loop then to heap allocate.
        if !self.is_sectors_cached(lba, amount) {
            return None;
        }

        let mut return_buffer = Vec::new();
        let mut offset = 0;

        for (sector, data) in self.cache.iter() {
            if *sector == (lba + offset) {
                for b in data.iter() {
                    return_buffer.push(*b);
                }

                offset += 1;
            }

            if offset == amount {
                return Some(return_buffer);
            }
        }

        None
    }

    pub fn is_sectors_cached(&self, lba: usize, amount: usize) -> bool {
        let mut offset = 0;
        for (sector, _) in self.cache.iter() {
            if *sector == (lba + offset) {
                offset += 1;
            }

            if offset == amount {
                return true;
            }
        }

        false
    }

    pub fn add_single_sector_to_cache(&mut self, lba: usize, data: &[u8]) {
        assert_eq!(data.len(), (Self::WORDS_PER_SECTOR * 2));

        let data_clone = Vec::new();
        for b in data {
            data_clone.push(*b);
        }

        self.cache.push((
                lba,
                data_clone
            ));
    }

    pub fn add_n_sectors_to_cache(&mut self, base_lba: usize, data: &[u8]) {
        assert_eq!(data.len() % (Self::WORDS_PER_SECTOR * 2), 0);

        data.s

    }

}

impl Display for ATADisk {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{} {:?} {:?} {:?} {} {}",
                 self.identify.model_number().as_str(),
                 self.identify.interconnect(),
                 self.identify.specific_config(),
                 self.identify.identify_completion_status(),
                 self.identify.max_sectors_per_request(),
                 Bytes::from(self.identify.user_sectors_28bit_lba() * 512)
        )?;

        Ok(())
    }
}

pub fn scan_for_disks() -> Vec<ATADisk> {
    let mut disks = Vec::new();

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
        while StatusRegister::is_busy(disk) &&
            !StatusRegister::is_err_or_fault(disk) &&
            !StatusRegister::is_status(disk, StatusFlags::DRQ) {}

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

        debug_println!("{}\t{}", "OK".green().bold(), disk.identify.model_number().as_str().dimmed());

        disks.push(disk);
    }

    disks
}

impl Medium for ATADisk {
    fn seek(&mut self, seek: SeekFrom) {
        match seek {
            SeekFrom::Start(start) => {
                self.current = start as usize;
            }
            SeekFrom::End(end) => {
                assert!(end <= 0);
                self.current = ((self.identify.user_sectors_28bit_lba() as i64) + end) as usize;
            }
            SeekFrom::Current(current) => {
                self.current = (current + (self.current as i64)) as usize;
            }
        }
    }

    fn read_exact(&self, amount: usize) -> Result<Vec<u8>, MediumErr> {
        todo!()
    }

    fn write_exact(&mut self, buf: Vec<u8>) -> Result<(), MediumErr> {
        todo!()
    }
}