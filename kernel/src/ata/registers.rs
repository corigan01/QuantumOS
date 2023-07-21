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

use core::slice::Iter;
use qk_alloc::vec::Vec;
use quantum_lib::x86_64::io::port::IOPort;
use quantum_utils::bitset::BitSet;

const PRIMARY_BUS_IO_BASE: usize = 0x1F0;
const SECONDARY_BUS_IO_BASE: usize = 0x170;

const PRIMARY_BUS_CONTROL_BASE: usize = 0x3F6;
const SECONDARY_BUS_CONTROL_BASE: usize = 0x376;

/// # R/W: Data Register (16-bit / 16-bit)
/// Read/Write PIO data bytes
const DATA_REGISTER_OFFSET_FROM_IO_BASE: usize = 0;

/// # R: Error Register (8-bit / 16-bit)
/// Used to retrieve any error generated by the last ATA command executed.
const ERROR_REGISTER_OFFSET_FROM_IO_BASE: usize = 1;

/// # W: Features Register (8-bit / 16-bit)
/// Used to control command specific interface features.
const FEATURES_REGISTER_OFFSET_FROM_IO_BASE: usize = 1;

/// # R/W: Sector Count Register (8-bit / 16-bit)
/// Number of sectors to read/write (0 is a special value).
const SECTOR_COUNT_OFFSET_FROM_IO_BASE: usize = 2;

/// # R/W: Sector Number Register (LBAlo) (8-bit / 16-bit)
/// This is CHS / LBA28 / LBA48 specific.
const SECTOR_NUM_LOW_OFFSET_FROM_IO_BASE: usize = 3;

/// # R/W: Cylinder Low Register / (LBAmid) (8-bit / 16-bit)
/// Partial Disk Sector address.
const SECTOR_NUM_MID_OFFSET_FROM_IO_BASE: usize = 4;

/// # R/W: Cylinder High Register / (LBAhi)	(8-bit / 16-bit)
/// Partial Disk Sector address.
const SECTOR_NUM_HIGH_OFFSET_FROM_IO_BASE: usize = 5;

/// # R/W: Drive / Head Register (8-bit / 8-bit)
/// Used to select a drive and/or head. Supports extra address/flag bits.
const DRIVE_HEAD_OFFSET_FROM_IO_BASE: usize = 6;

/// # R: Status Register (8-bit / 8-bit)
/// Used to read the current status.
const STATUS_REGISTER_OFFSET_FROM_IO_BASE: usize = 7;

/// # W: Command Register (8-bit / 8-bit)
/// Used to send ATA commands to the device.
const COMMAND_OFFSET_FROM_IO_BASE: usize = 7;

/// # R: Alternate Status Register (8-bit / 8-bit)
/// A duplicate of the Status Register which does not affect interrupts.
const ALTERNATE_STATUS_REGISTER_OFFSET_FROM_CONTROL_BASE: usize = 0;

/// # W: Device Control Register (8-bit / 8-bit)
/// Used to reset the bus or enable/disable interrupts.
const DEVICE_CONTROL_REGISTER_OFFSET_FROM_CONTROL_BASE: usize = 0;

/// # R: Drive Address Register (8-bit / 8-bit)
/// Provides drive select and head select information.
const DRIVE_ADDRESS_REGISTER_OFFSET_FROM_CONTROL_BASE: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DiskID {
    PrimaryFirst,
    PrimarySecond,
    SecondaryFirst,
    SecondarySecond
}

impl DiskID {
    const ALL_DISKS: [DiskID; 4] = [DiskID::PrimaryFirst, DiskID::PrimarySecond, DiskID::SecondaryFirst, DiskID::SecondarySecond];

    pub fn iter() -> Iter<'static, DiskID> {
        Self::ALL_DISKS.iter()
    }

    pub const fn bus_base(&self) -> usize {
        match self {
            DiskID::PrimaryFirst => PRIMARY_BUS_IO_BASE,
            DiskID::PrimarySecond => PRIMARY_BUS_IO_BASE,
            DiskID::SecondaryFirst => SECONDARY_BUS_IO_BASE,
            DiskID::SecondarySecond => SECONDARY_BUS_IO_BASE,
        }
    }

    pub const fn control_base(&self) -> usize {
        match self {
            DiskID::PrimaryFirst => PRIMARY_BUS_CONTROL_BASE,
            DiskID::PrimarySecond => PRIMARY_BUS_CONTROL_BASE,
            DiskID::SecondaryFirst => SECONDARY_BUS_CONTROL_BASE,
            DiskID::SecondarySecond => SECONDARY_BUS_CONTROL_BASE,
        }
    }

    pub fn is_first(&self) -> bool {
        match self {
            DiskID::PrimaryFirst => true,
            DiskID::PrimarySecond => false,
            DiskID::SecondaryFirst => true,
            DiskID::SecondarySecond => false
        }
    }

    pub fn is_second(&self) -> bool {
        !self.is_first()
    }
}

pub enum StatusFlags {
    /// Indicates an error occurred. Send a new command to clear it (or nuke it with a Software Reset).
    Err,
    /// Index. Always set to zero.
    Idx,
    /// Corrected data. Always set to zero.
    CorrectedData,
    /// Set when the drive has PIO data to transfer, or is ready to accept PIO data.
    DRQ,
    /// Overlapped Mode Service Request.
    SRV,
    /// Drive Fault Error (**does not set ERR**).
    DriveFault,
    /// Bit is clear when drive is spun down, or after an error. Set otherwise.
    SpinDown,
    /// Indicates the drive is preparing to send/receive data (wait for it to clear).
    /// In case of 'hang' (it never clears), do a software reset.
    Busy
}

pub struct StatusRegister {}
impl StatusRegister {
    const ATA_SR_BSY_BIT: u8 = 7;
    const ATA_SR_DRDY_BIT: u8 = 6;
    const ATA_SR_DF_BIT: u8 = 5;
    const ATA_SR_DSC_BIT: u8 = 4;
    const ATA_SR_DRQ_BIT: u8 = 3;
    const ATA_SR_CORR_BIT: u8 = 2;
    const ATA_SR_IDX_BIT: u8 = 1;
    const ATA_SR_ERR_BIT: u8 = 0;

    fn my_port(device: DiskID) -> IOPort {
        let device_io = device.bus_base() + STATUS_REGISTER_OFFSET_FROM_IO_BASE;

        IOPort::new(device_io as u16)
    }

    pub fn read(device: DiskID) -> u8 {
        let my_port = Self::my_port(device);

        unsafe { my_port.read_u8() }
    }

    pub fn perform_400ns_delay(device: DiskID) {
        let my_port = Self::my_port(device);

        for _ in 0..15 {
            unsafe { my_port.read_u8() };
        }
    }

    pub fn is_floating(device: DiskID) -> bool {
        let read = Self::read(device);
        read == 0
    }

    pub fn is_status(device: DiskID, status: StatusFlags) -> bool {
        let read_value = Self::read(device);

        let bit = match status {
            StatusFlags::Err => Self::ATA_SR_ERR_BIT,
            StatusFlags::Idx => Self::ATA_SR_IDX_BIT,
            StatusFlags::CorrectedData => Self::ATA_SR_CORR_BIT,
            StatusFlags::DRQ => Self::ATA_SR_DRQ_BIT,
            StatusFlags::SRV => Self::ATA_SR_DRDY_BIT,
            StatusFlags::DriveFault => Self::ATA_SR_DF_BIT,
            StatusFlags::SpinDown => Self::ATA_SR_DSC_BIT,
            StatusFlags::Busy => Self::ATA_SR_BSY_BIT
        };

        read_value.get_bit(bit)
    }

    pub fn is_err_or_fault(device: DiskID) -> bool {
        Self::is_status(device, StatusFlags::Err) ||
            Self::is_status(device, StatusFlags::DriveFault)
    }

    pub fn is_busy(device: DiskID) -> bool {
        Self::is_status(device, StatusFlags::Busy)
    }

}

#[derive(Debug, Clone, Copy)]
pub enum ErrorFlags {
    /// 0:	(AMNF)    Address mark not found.
    AddressMarkNotFound,
    /// 1:	(TKZNF)   Track zero not found.
    TrackZeroNotFound,
    /// 2:	(ABRT)    Aborted command.
    AbortedCommand,
    /// 3:	(MCR)	  Media change request.
    MediaChangeRequest,
    /// 4:	(IDNF)    ID not found.
    IDNotFound,
    /// 5:	(MC)      Media changed.
    MediaChanged,
    /// 6:	(UNC)     Uncorrectable data error
    UncorrectableDataError,
    /// 7:	(BBK)     Bad Block detected.
    BadBlockDetected,
}

impl ErrorFlags {
    const ERR_CONST: [ErrorFlags; 8] = [
        ErrorFlags::AddressMarkNotFound,
        ErrorFlags::TrackZeroNotFound,
        ErrorFlags::AbortedCommand,
        ErrorFlags::MediaChangeRequest,
        ErrorFlags::IDNotFound,
        ErrorFlags::MediaChanged,
        ErrorFlags::UncorrectableDataError,
        ErrorFlags::BadBlockDetected
    ];

    pub fn iter() -> Iter<'static, ErrorFlags> {
        Self::ERR_CONST.iter()
    }
}

pub struct ErrorRegister {}
impl ErrorRegister {
    const ATA_ER_BBK_BIT: u8 = 7;
    const ATA_ER_UNC_BIT: u8 = 6;
    const ATA_ER_MC_BIT: u8 = 5;
    const ATA_ER_IDNF_BIT: u8 = 4;
    const ATA_ER_MCR_BIT: u8 = 3;
    const ATA_ER_ABRT_BIT: u8 = 2;
    const ATA_ER_TKONF_BIT: u8 = 1;
    const ATA_ER_AMNF_BIT: u8 = 0;

    fn my_port(device: DiskID) -> IOPort {
        let io_port = device.bus_base() + ERROR_REGISTER_OFFSET_FROM_IO_BASE;

        IOPort::new(io_port as u16)
    }

    pub fn read(device: DiskID) -> u8 {
        let port = Self::my_port(device);

        unsafe { port.read_u8() }
    }

    pub fn any_error(device: DiskID) -> bool {
        Self::read(device) > 0
    }

    pub fn is_error(device: DiskID, error: ErrorFlags) -> bool {
        let value = Self::read(device);

        if value == 0 {
            return false;
        }

        match error {
            ErrorFlags::AddressMarkNotFound => {
                value & (1 << Self::ATA_ER_AMNF_BIT) != 0
            }
            ErrorFlags::TrackZeroNotFound => {
                value & (1 << Self::ATA_ER_TKONF_BIT) != 0
            }
            ErrorFlags::AbortedCommand => {
                value & (1 << Self::ATA_ER_ABRT_BIT) != 0
            }
            ErrorFlags::MediaChangeRequest => {
                value & (1 << Self::ATA_ER_MCR_BIT) != 0
            }
            ErrorFlags::IDNotFound => {
                value & (1 << Self::ATA_ER_IDNF_BIT) != 0
            }
            ErrorFlags::MediaChanged => {
                value & (1 << Self::ATA_ER_MC_BIT) != 0
            }
            ErrorFlags::UncorrectableDataError => {
                value & (1 << Self::ATA_ER_UNC_BIT) != 0
            }
            ErrorFlags::BadBlockDetected => {
                value & (1 << Self::ATA_ER_BBK_BIT) != 0
            }
        }
    }

    pub fn all_flags(device: DiskID) -> Vec<ErrorFlags> {
        let mut errors = Vec::new();

        for flags in ErrorFlags::iter() {
            if Self::is_error(device, *flags) {
                errors.push(*flags);
            }
        }

        errors
    }
}

static mut LAST_DISK: Option<DiskID> = None;
pub struct DriveHeadRegister {}
impl DriveHeadRegister {
    const ATA_DH_DRV: u8 = 4;
    const ATA_DH_RESERVED_1: u8 = 5;
    const ATA_DH_LBA: u8 = 6;
    const ATA_DH_RESERVED_2: u8 = 7;

    fn my_port(device: DiskID) -> IOPort {
        let io_port = device.bus_base() + DRIVE_HEAD_OFFSET_FROM_IO_BASE;

        IOPort::new(io_port as u16)
    }

    pub fn read(device: DiskID) -> u8 {
        let port = Self::my_port(device);

        unsafe { port.read_u8() }
    }

    pub fn write(device: DiskID, value: u8) {
        let port = Self::my_port(device);

        unsafe { port.write_u8(value) };
    }

    pub fn is_using_chs(device: DiskID) -> bool {
        let value = Self::read(device);

        value & (1 << Self::ATA_DH_LBA) == 0
    }

    pub fn set_bits_24_to_27_of_lba(device: DiskID, lba_bits: u8) {
        if lba_bits & 0b11111000 > 0 {
            panic!("Should not be sending more then 3 bits to DriveHeadRegister!");
        }

        let read_value = Self::read(device);
        Self::write(device, (read_value & 0b11111000) | lba_bits);
    }

    pub fn is_using_lba(device: DiskID) -> bool {
        !Self::is_using_chs(device)
    }

    pub fn clear_select_disk_cache() {
        unsafe { LAST_DISK = None };
    }

    pub fn note_disk(device: DiskID) {
        unsafe { LAST_DISK = Some(device) };
    }

    fn should_skip(device: DiskID) -> bool {
        unsafe {
            if LAST_DISK.is_none() {
                return false;
            }

            LAST_DISK.unwrap() == device
        }
    }

    pub fn select_drive(device: DiskID) {
        if Self::should_skip(device) {
            return;
        }

        let mut read = Self::read(device);
        let write_value = read.set_bit(Self::ATA_DH_DRV, device.is_second());
        Self::write(device, write_value);
    }
}

pub struct SectorRegisters {}
impl SectorRegisters {
    const SECTOR_COUNT: usize = 0;
    const SECTOR_LOW: usize = 1;
    const SECTOR_MID: usize = 2;
    const SECTOR_HIGH: usize = 3;

    const IO_PORT_OFFSETS: [usize; 4] = [
        SECTOR_COUNT_OFFSET_FROM_IO_BASE,
        SECTOR_NUM_LOW_OFFSET_FROM_IO_BASE,
        SECTOR_NUM_MID_OFFSET_FROM_IO_BASE,
        SECTOR_NUM_HIGH_OFFSET_FROM_IO_BASE
    ];

    fn my_port(device: DiskID, select: usize) -> IOPort {
        let port = device.bus_base() + Self::IO_PORT_OFFSETS[select];
        IOPort::new(port as u16)
    }

    pub fn zero_registers(device: DiskID)  {
        for offset in Self::IO_PORT_OFFSETS {
            let port = offset + device.bus_base();
            let io_port = IOPort::new(port as u16);

            unsafe { io_port.write_u8(0) };
        }
    }

    pub fn are_all_zero(device: DiskID) -> bool {
        for offset in Self::IO_PORT_OFFSETS {
            let port = offset + device.bus_base();
            let io_port = IOPort::new(port as u16);

            if unsafe { io_port.read_u8() } != 0 {
                return false;
            }
        }

        true
    }

    pub fn select_sectors(device: DiskID, sectors: u8) {
        let port = Self::my_port(device, Self::SECTOR_COUNT);

        unsafe { port.write_u8(sectors) };
    }

    pub fn select_lba_0_to_24_bits(device: DiskID, lba: usize) {
        let low = Self::my_port(device, Self::SECTOR_LOW);
        let mid = Self::my_port(device, Self::SECTOR_MID);
        let high = Self::my_port(device, Self::SECTOR_HIGH);

        unsafe {
            low.write_u8((lba & 0xFF) as u8);
            mid.write_u8(((lba >> 8) & 0xFF) as u8);
            high.write_u8(((lba >> 16) & 0xFF) as u8);
        }
    }
}

pub struct FeaturesRegister {}
impl FeaturesRegister {
    fn my_port(device: DiskID) -> IOPort {
        let port = device.bus_base() + FEATURES_REGISTER_OFFSET_FROM_IO_BASE;
        IOPort::new(port as u16)
    }

    fn write(device: DiskID, value: u8) {
        let port = Self::my_port(device);

        unsafe { port.write_u8(value) };
    }

    pub fn reset_to_zero(device: DiskID) {
        Self::write(device, 0);
    }
}

pub enum Commands {
    Identify,
    ReadSectorsPIO,
    WriteSectorsPIO,
    CacheFlush
}

pub struct CommandRegister {}
impl CommandRegister {
    const ATA_CMD_READ_PIO: u8 = 0x20;
    const ATA_CMD_READ_PIO_EXT: u8 = 0x24;
    const ATA_CMD_READ_DMA: u8 = 0xC8;
    const ATA_CMD_READ_DMA_EXT: u8 = 0x25;
    const ATA_CMD_WRITE_PIO: u8 = 0x30;
    const ATA_CMD_WRITE_PIO_EXT: u8 = 0x34;
    const ATA_CMD_WRITE_DMA: u8 = 0xCA;
    const ATA_CMD_WRITE_DMA_EXT: u8 = 0x35;
    const ATA_CMD_CACHE_FLUSH: u8 = 0xE7;
    const ATA_CMD_CACHE_FLUSH_EXT: u8 = 0xEA;
    const ATA_CMD_PACKET: u8 = 0xA0;
    const ATA_IDENTIFY_PACKET: u8 = 0xA1;
    const ATA_IDENTIFY: u8 = 0xEC;

    pub fn send_command(device: DiskID, command: Commands) {
        let port = device.bus_base() + COMMAND_OFFSET_FROM_IO_BASE;
        let io_port = IOPort::new(port as u16);

        let command_id = match command {
            Commands::Identify => Self::ATA_IDENTIFY,
            Commands::ReadSectorsPIO => Self::ATA_CMD_READ_PIO,
            Commands::WriteSectorsPIO => Self::ATA_CMD_WRITE_PIO,
            Commands::CacheFlush => Self::ATA_CMD_CACHE_FLUSH
        };

        unsafe { io_port.write_u8(command_id) };
    }
}

pub struct DataRegister {}
impl DataRegister {

    fn my_port(device: DiskID) -> IOPort {
        let port = device.bus_base() + DATA_REGISTER_OFFSET_FROM_IO_BASE;
        IOPort::new(port as u16)
    }

    pub fn read_u16(device: DiskID) -> u16 {
        let io_port = Self::my_port(device);

        unsafe { io_port.read_u16() }
    }

    pub fn read_u8(device: DiskID) -> u16 {
        let io_port = Self::my_port(device);

        unsafe { io_port.read_u16() }
    }

    pub fn write_u16(device: DiskID, value: u16) {
        let my_port = Self::my_port(device);

        unsafe { my_port.write_u16(value) };
    }

    pub fn write_u8(device: DiskID, value: u8) {
        let my_port = Self::my_port(device);

        unsafe { my_port.write_u8(value) };
    }

}