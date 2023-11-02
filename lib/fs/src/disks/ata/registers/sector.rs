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
    SECTOR_COUNT_OFFSET_FROM_IO_BASE, SECTOR_NUM_HIGH_OFFSET_FROM_IO_BASE,
    SECTOR_NUM_LOW_OFFSET_FROM_IO_BASE, SECTOR_NUM_MID_OFFSET_FROM_IO_BASE,
};
use core::marker::PhantomData;

pub struct SectorNone {}
pub struct SectorCount {}
pub struct SectorLow {}
pub struct SectorMid {}
pub struct SectorHigh {}

pub struct SectorRegister<RegisterSelect = SectorNone>(PhantomData<RegisterSelect>);

impl ResolveIOPortBusOffset<SECTOR_COUNT_OFFSET_FROM_IO_BASE> for SectorRegister<SectorCount> {}
impl ResolveIOPortBusOffset<SECTOR_NUM_LOW_OFFSET_FROM_IO_BASE> for SectorRegister<SectorLow> {}
impl ResolveIOPortBusOffset<SECTOR_NUM_MID_OFFSET_FROM_IO_BASE> for SectorRegister<SectorMid> {}
impl ResolveIOPortBusOffset<SECTOR_NUM_HIGH_OFFSET_FROM_IO_BASE> for SectorRegister<SectorHigh> {}

unsafe impl ReadRegisterBus<SECTOR_COUNT_OFFSET_FROM_IO_BASE> for SectorRegister<SectorCount> {}
unsafe impl ReadRegisterBus<SECTOR_NUM_LOW_OFFSET_FROM_IO_BASE> for SectorRegister<SectorLow> {}
unsafe impl ReadRegisterBus<SECTOR_NUM_MID_OFFSET_FROM_IO_BASE> for SectorRegister<SectorMid> {}
unsafe impl ReadRegisterBus<SECTOR_NUM_HIGH_OFFSET_FROM_IO_BASE> for SectorRegister<SectorHigh> {}

unsafe impl WriteRegisterBus<SECTOR_COUNT_OFFSET_FROM_IO_BASE> for SectorRegister<SectorCount> {}
unsafe impl WriteRegisterBus<SECTOR_NUM_LOW_OFFSET_FROM_IO_BASE> for SectorRegister<SectorLow> {}
unsafe impl WriteRegisterBus<SECTOR_NUM_MID_OFFSET_FROM_IO_BASE> for SectorRegister<SectorMid> {}
unsafe impl WriteRegisterBus<SECTOR_NUM_HIGH_OFFSET_FROM_IO_BASE> for SectorRegister<SectorHigh> {}

impl SectorRegister {
    /// # Zero All Registers
    /// Zeros all the sector registers. Zeroing the registers is a special value.
    pub unsafe fn zero_all_registers(disk: DiskID) {
        SectorRegister::<SectorCount>::write(disk, 0);
        SectorRegister::<SectorLow>::write(disk, 0);
        SectorRegister::<SectorMid>::write(disk, 0);
        SectorRegister::<SectorHigh>::write(disk, 0);
    }

    /// # Is Registers Zeros?
    /// Checks if the registers are zeroed.
    pub fn is_registers_zeroed(disk: DiskID) -> bool {
        if SectorRegister::<SectorCount>::read(disk) != 0 {
            return false;
        }

        if SectorRegister::<SectorLow>::read(disk) != 0 {
            return false;
        }

        if SectorRegister::<SectorMid>::read(disk) != 0 {
            return false;
        }

        if SectorRegister::<SectorHigh>::read(disk) != 0 {
            return false;
        }

        true
    }

    /// # Set Sectors per Operation
    /// Selects how many sectors the disk is going to process at one time. When reading/writting
    /// this selects how many sectors its going to read/write.
    pub fn set_sectors_per_operation(device: DiskID, sectors: u8) {
        unsafe { SectorRegister::<SectorCount>::write(device, sectors) }
    }

    /// # LBA Bits 0 to 24
    ///
    /// # Panics
    /// Asserts to ensure that only the lower 24 bits are set.
    pub fn lba_bits_0_24(device: DiskID, lba_bits: usize) {
        assert!(
            lba_bits & 0xFFFFFF == lba_bits,
            "LBA Bits should not be larger then 24 bits!"
        );

        unsafe {
            SectorRegister::<SectorLow>::write(device, (lba_bits & 0xFF) as u8);
            SectorRegister::<SectorMid>::write(device, ((lba_bits >> 8) & 0xFF) as u8);
            SectorRegister::<SectorHigh>::write(device, ((lba_bits >> 16) & 0xFF) as u8);
        }
    }
}
