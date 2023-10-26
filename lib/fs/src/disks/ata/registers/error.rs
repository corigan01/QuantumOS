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

use super::{DiskID, ReadRegisterBus, ResolveIOPortBusOffset, ERROR_REGISTER_OFFSET_FROM_IO_BASE};
use core::{fmt::Debug, slice::Iter};

/// # Error Flags
/// The possible flags the error register could return. Use these to check if the error
/// register contains an error.
#[derive(Debug, Clone, Copy)]
pub enum ErrorFlags {
    /// 0: (AMNF) Address mark not found.
    AddressMarkNotFound,
    /// 1: (TKZNF) Track zero not found.
    TrackZeroNotFound,
    /// 2: (ABRT) Aborted Command.
    AbortedCommand,
    /// 3: (MCR) Media Change requested.
    MediaChangeRequest,
    /// 4 (IDNF) ID Not Found.
    IDNotFound,
    /// 5 (MC) Media Changed.
    MediaChanged,
    /// 6 (UNC) Uncorrectable data error.
    UncorrectableDataError,
    /// 7 (BBk) Bad Block detected.
    BadBlockDetected,
}

impl ErrorFlags {
    /// # Error Flags
    /// Contains a const slice of errors for use in making an iterator
    const ERROR_FLAGS: [Self; 8] = [
        Self::AddressMarkNotFound,
        Self::TrackZeroNotFound,
        Self::AbortedCommand,
        Self::MediaChangeRequest,
        Self::IDNotFound,
        Self::MediaChanged,
        Self::UncorrectableDataError,
        Self::BadBlockDetected,
    ];

    /// # Into Bit Mask
    /// Converts the Error Flag values into their bit mask value.
    pub const fn into_bit_mask(&self) -> u8 {
        1 << match *self {
            Self::AddressMarkNotFound => 0,
            Self::TrackZeroNotFound => 1,
            Self::AbortedCommand => 2,
            Self::MediaChangeRequest => 3,
            Self::IDNotFound => 4,
            Self::MediaChanged => 5,
            Self::UncorrectableDataError => 6,
            Self::BadBlockDetected => 7,
        }
    }

    /// # Iter
    /// Returns an iterator of all the possible ErrorFlags. Iterator is in order of
    /// how the elements appear in the enum.
    pub fn iter() -> Iter<'static, Self> {
        Self::ERROR_FLAGS.iter()
    }
}

/// # Error Field Flags
/// Contains all the possible errors that the register could be holding. Each bit
/// represents an ErrorFlag. Simply a strongly typed u8 to store the errors.
#[derive(Clone, Copy)]
pub struct ErrorFieldFlags(u8);

impl ErrorFieldFlags {
    /// # Check Flag
    /// Chekcs if a flag is present in the error register.
    pub fn check_flag(&self, flag: ErrorFlags) -> bool {
        let mask = flag.into_bit_mask();

        self.0 & mask != 0
    }

    /// # Is Error?
    /// Checks if any error is present.
    pub fn is_error(&self) -> bool {
        self.0 != 0
    }
}

impl Debug for ErrorFieldFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("ErrorFieldFlags: ")?;
        let mut entry_list = f.debug_list();

        for flag in ErrorFlags::iter() {
            if self.check_flag(*flag) {
                entry_list.entry(flag);
            }
        }

        entry_list.finish()
    }
}

/// # Error Register
/// Drive error register. Disk errors and other fault states are stored here.
pub struct ErrorRegister {}
impl ResolveIOPortBusOffset<ERROR_REGISTER_OFFSET_FROM_IO_BASE> for ErrorRegister {}
unsafe impl ReadRegisterBus<ERROR_REGISTER_OFFSET_FROM_IO_BASE> for ErrorRegister {}

impl ErrorRegister {
    /// # Get Errors
    /// Returns an ErrorFieldFlags with all errors detected in the register. Use
    /// ErrorFieldFlags to check error state.
    pub fn get_errors(disk_id: DiskID) -> ErrorFieldFlags {
        ErrorFieldFlags(Self::read(disk_id))
    }
}
