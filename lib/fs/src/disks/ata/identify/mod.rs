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

use self::raw::RawIdentifyStruct;
use qk_alloc::{boxed::Box, string::String};
use quantum_lib::bitset::BitSet;

mod raw;

/// # Identify Parser
/// The recommended way of interacting with the identify struct. The identify struct can be
/// complicated as many of the fields are embeded as bit flags in weirdly named fields. This struct
/// provides methods for interacting with this functionality in a more trasparent and easy to use
/// way.
pub struct IdentifyParser {
    raw_identify: Box<RawIdentifyStruct>,
}

impl IdentifyParser {
    /// # Logical Sector Size
    /// Gets the sector size of a logical sector on the disk.
    pub fn logical_sector_size(&self) -> usize {
        match self.raw_identify.logical_sector_size as usize {
            0 => 512,
            n => n,
        }
    }

    /// # Empty
    /// Gives a empty box with zeroed data. Useful for making a IdentifyParser before you have the
    /// data. Mostly just a patch for AtaDisk.
    pub unsafe fn empty() -> Self {
        Self {
            raw_identify: Box::new_zeroed().assume_init(),
        }
    }

    /// # User Sectors 28bit LBA
    /// Gets the sectors that are accessable on the disk in 28 bit lba mode.
    pub fn user_sectors_28bit_lba(&self) -> usize {
        self.raw_identify.user_addressable_logical_sectors_lba28 as usize
    }

    /// # Max Sectors Per DRQ Request
    /// The max sectors the disk can process at one time per drq request. This is complicated
    /// because it seems that in PIO mode (on qemu) this isn't really a bit deal as you can still
    /// process more. However, on real hardware who knows if the drive is going to be up to spec,
    /// so I would recommend this to be used when making requests to the drive.
    pub fn max_sectors_per_drq_request(&self) -> usize {
        (self.raw_identify.logical_sectors_per_drq & 0xFF) as usize
    }

    /// # Is 48bit LBA supported?
    /// Checks if 48bit lba mode is supported on this drive.
    pub fn is_48_bit_lba_supported(&self) -> bool {
        let lba_support = self
            .raw_identify
            .commands_and_feature_sets_supported_or_enabled2;

        lba_support.get_bit(10)
    }

    /// # Model Number
    /// Gets the model string from the drive. This is not really a number, but more of a string
    /// that identifies the drive from others. May contain numbers, but letters or words are valid
    /// per spec.
    pub fn model_number(&self) -> String {
        // FIXME: This could be optimized in the future, however, getting the Model Number of a
        // disk is not a thing that happens very often.
        let mut string = String::new();
        let model_number = self.raw_identify.model_number;

        'outer: for word in model_number {
            let bytes = word.to_be_bytes();

            for byte in bytes {
                if byte == 0 || byte == 16 {
                    break 'outer;
                }

                if !byte.is_ascii() {
                    continue;
                }

                string.push(byte as char);
            }
        }

        String::from(string.trim())
    }
}

impl From<&[u8]> for IdentifyParser {
    fn from(value: &[u8]) -> Self {
        let raw_identify = RawIdentifyStruct::from(value);

        Self {
            raw_identify: Box::new(raw_identify),
        }
    }
}
