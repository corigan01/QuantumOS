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

use super::{
    DiskID, ReadRegisterBus, ResolveIOPortBusOffset, WriteRegisterBus,
    DRIVE_HEAD_OFFSET_FROM_IO_BASE,
};
use quantum_lib::bitset::BitSet;

/// # Last Disk
/// Switching disks can take a large amount of time, so if we cache which disk was used
/// last then it will speed up access times.
static mut LAST_DISK: Option<DiskID> = None;

/// # Drive Head Register
/// ATA PIO Register that controls the drive head including some sector info, and
/// which disk is selected for reading/writing. Drive Head Register also
/// selects/stores with access method the disk is using (LBA or CHS).
pub struct DriveHeadRegister {}
impl ResolveIOPortBusOffset<DRIVE_HEAD_OFFSET_FROM_IO_BASE> for DriveHeadRegister {}
unsafe impl ReadRegisterBus<DRIVE_HEAD_OFFSET_FROM_IO_BASE> for DriveHeadRegister {}
unsafe impl WriteRegisterBus<DRIVE_HEAD_OFFSET_FROM_IO_BASE> for DriveHeadRegister {}

impl DriveHeadRegister {
    const ATA_DRV: u8 = 4;
    const ATA_LBA: u8 = 6;

    /// # Is using CHS?
    /// Checks if the disk selected is using CHS addressing
    pub fn is_using_chs(device: DiskID) -> bool {
        (Self::read(device) & (1 << Self::ATA_LBA)) == 0
    }

    /// # Lba Bits 24 to 27
    /// Sets the upper 3 bits of the lba. This is quite a weird spot to do such
    /// a thing, but its how the drive expects us to set it.
    ///
    /// # Panics
    /// Asserts to ensure only the bottom 3 bits are set. If a higher bit it set, then
    /// this function will panic.
    pub fn lba_bits_24_27(device: DiskID, lba_bits: u8) {
        assert!(
            lba_bits & 0b11111000 == lba_bits,
            "Should not be sending more then 3 bits to DriveHeadRegister"
        );

        let read_reg = Self::read(device);
        unsafe { Self::write(device, (read_reg & !0b111) | lba_bits) }
    }

    /// # Is using LBA?
    /// Checks if the disk is using LBA addressing mode.
    pub fn is_using_lba(device: DiskID) -> bool {
        !Self::is_using_chs(device)
    }

    /// # Clear Select Cache
    /// Resets the last disk used in the cache. Ensures the next switch operation
    /// will fully select the disk.
    pub fn clear_select_cache() {
        unsafe { LAST_DISK = None }
    }

    /// # Acknowledge Disk
    /// Sets the last disk cache to make sure if we attempt to select this disk again
    /// it will not do a double operation.
    pub fn acknowledge_disk(disk: DiskID) {
        unsafe { LAST_DISK = Some(disk) }
    }

    /// # Require Disk Switch?
    /// Checks if the current disk is the same one that was previously selected.
    pub fn require_disk_switch(disk: DiskID) -> bool {
        unsafe {
            LAST_DISK
                .map(|last_disk| last_disk == disk)
                .unwrap_or(false)
        }
    }

    /// # Force Switch Disk
    /// Does the raw operation to change disks, does not influence the cache or check it.
    /// This is unsafe because it does not update the cache and could cause the next disk
    /// switch to skip, thus causing the wrong disk to be read/written.
    pub unsafe fn force_switch_disk(disk: DiskID) {
        let mut read_reg = Self::read(disk);
        let new_bit = read_reg.set_bit(Self::ATA_DRV, disk.is_second());
        Self::write(disk, new_bit);
    }

    /// # Switch Disk
    /// Only switches the disk if the last selected disk is different from the current one.
    ///
    /// Selects the disk that DiskID points to.
    pub fn switch_disk(disk: DiskID) -> bool {
        if !Self::require_disk_switch(disk) {
            return false;
        }

        Self::acknowledge_disk(disk);
        unsafe { Self::force_switch_disk(disk) };

        true
    }
}
