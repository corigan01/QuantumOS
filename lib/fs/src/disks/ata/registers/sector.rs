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
    DiskID, IOPortOffset, ReadRegisterBus, ResolveIOPortBusOffset, WriteRegisterBus,
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
