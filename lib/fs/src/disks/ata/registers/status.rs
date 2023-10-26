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

use super::{DiskID, ReadRegisterBus, ResolveIOPortBusOffset, STATUS_REGISTER_OFFSET_FROM_IO_BASE};
use core::{fmt::Debug, slice::Iter};

/// # Status Flags
/// Possible flags that the status register could contain. Use these to check the status with
/// StatusFieldFlags.
#[derive(Debug, Clone, Copy)]
pub enum StatusFlags {
    /// Indicates an error has occurred. Send a new command to clear it (or remove it with a
    /// software reset)
    ErrorOccurred,
    /// Index -- always set to zero.
    Index,
    /// Corrected Data -- always set to zero.
    CorrectedData,
    /// Set when the drive has PIO data to transfer, or when its ready to accept PIO data.
    Ready,
    /// Overlapping service requests.
    TooManyServiceReqests,
    /// Drive fault error -- does not set ErrorRegister!
    DriveFault,
    /// Clear when disk is spun down, also is clear after an error. It should always be set for PIO
    /// HHDs when successfully reading and writting.
    SpinDown,
    /// Indicates that the drive is preparing to send/receive data. Waiting is often the best way
    /// of handing this flag. For whatever reason, if the drive does not clear this flag on its own
    /// i.e the disk is 'hanging', a software reset should be sent to the drive.
    Busy,
}

impl StatusFlags {
    /// # Status Flags
    /// A const slice to store the values for an iterator.
    const STATUS_FLAGS: [Self; 8] = [
        Self::ErrorOccurred,
        Self::Index,
        Self::CorrectedData,
        Self::Ready,
        Self::TooManyServiceReqests,
        Self::DriveFault,
        Self::SpinDown,
        Self::Busy,
    ];

    /// # Into Bit Mask
    /// Creates a bit mask for each entry in the enum. Each flag is really just a bit that is set
    /// on the status register, so we select the same bit here so we can represent it as a enum.
    pub fn into_bit_mask(&self) -> u8 {
        1 << match *self {
            Self::ErrorOccurred => 0,
            Self::Index => 1,
            Self::CorrectedData => 2,
            Self::Ready => 3,
            Self::TooManyServiceReqests => 4,
            Self::DriveFault => 5,
            Self::SpinDown => 6,
            Self::Busy => 7,
        }
    }

    /// # Iter
    /// Returns an iterator over the elements in the enum. The iterator is in the same order as the
    /// elements appear in the enum.
    pub fn iter() -> Iter<'static, Self> {
        Self::STATUS_FLAGS.iter()
    }
}

/// # Status Field Flags
/// Value returned from StatusRegister that stores all status infomation about the disk. Use
/// StatusFlags to check if a flag is set.
#[derive(Clone, Copy)]
pub struct StatusFieldFlags(u8);

impl StatusFieldFlags {
    /// # Check Flag
    /// Checks if a flag is set/unset from StatusFlags. Returns state of the bit.
    pub fn check_flag(&self, flag: StatusFlags) -> bool {
        self.0 & flag.into_bit_mask() != 0
    }

    /// # Is Status?
    /// Checks if the status register is not zero.
    pub fn is_status(&self) -> bool {
        self.0 > 0
    }
}

impl Debug for StatusFieldFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("StatusFieldFlags: ")?;
        let mut list = f.debug_list();

        for flag in StatusFlags::iter() {
            if self.check_flag(*flag) {
                list.entry(flag);
            }
        }

        list.finish()
    }
}

/// # Status Register
/// Status infomation about a disk drive.
pub struct StatusRegister {}
impl ResolveIOPortBusOffset<STATUS_REGISTER_OFFSET_FROM_IO_BASE> for StatusRegister {}
unsafe impl ReadRegisterBus<STATUS_REGISTER_OFFSET_FROM_IO_BASE> for StatusRegister {}

impl StatusRegister {
    /// # Get Status
    /// Gets the status of the register. Use StatusFlags to check if a flag is set or not in the
    /// StatusFieldFlags value.
    pub fn get_status(disk: DiskID) -> StatusFieldFlags {
        StatusFieldFlags(Self::read(disk))
    }

    /// # Is Bus Floating?
    /// Checks if the bus is in a state of floating. Floating useally occurs when the bus is not
    /// connected. The best way to detect if the bus is present is to check this. Since you know a
    /// bus is not present you know no disks are connected on that bus and can save a lot of time.
    pub fn is_bus_floating(disk: DiskID) -> bool {
        Self::read(disk) == 0
    }

    /// # Preform 400ns Delay
    /// Sleeps 400ns by reading the IOBus 16 times. The drive is really slow compared to our CPU,
    /// so even with the IO bus being slower then the CPU, the drive still needs some time to
    /// respond to our requests. This is often used when asking the disk to complete some task that
    /// might take its controller a bit to process. CPU == brr -- Disk == Slllooooowwww.
    pub fn perform_400ns_delay(disk: DiskID) {
        for _ in 0..16 {
            let _ = Self::read(disk);
        }
    }
}
