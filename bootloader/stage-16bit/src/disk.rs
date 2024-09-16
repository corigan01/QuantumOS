/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
    Part of the Quantum OS Project

Copyright 2024 Gavin Kellam

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

use bios::disk::raw_read;
use bios::BiosStatus;
use fs::read_block::BlockDevice;

use fs::error::{FsError, Result};
use fs::io::{Read, Seek, SeekFrom};

#[link_section = ".buffer"]
static mut TEMP_BUFFER: [u8; 512] = [0u8; 512];

pub struct BiosDisk {
    id: u16,
    seek: u64,
}

impl BiosDisk {
    pub fn new(disk_id: u16) -> Self {
        Self {
            id: disk_id,
            seek: 0,
        }
    }
}

impl BlockDevice for BiosDisk {
    const BLOCK_SIZE: usize = 512;

    fn read_block<'a>(&'a mut self, block_offset: u64) -> Result<&'a [u8]> {
        match unsafe { raw_read(self.id, block_offset, 1, TEMP_BUFFER.as_mut_ptr()) } {
            BiosStatus::Success => Ok(unsafe { TEMP_BUFFER.as_slice() }),
            BiosStatus::InvalidInput | BiosStatus::InvalidData => Err(FsError::InvalidInput),
            BiosStatus::NotSupported => Err(FsError::NotSupported),
            BiosStatus::Failed => Err(FsError::ReadError),
        }
    }
}

impl Read for BiosDisk {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        fs::read_block::read_smooth_from_block_device(self, self.seek, buf)
    }
}

impl Seek for BiosDisk {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            SeekFrom::Start(start) => {
                self.seek = start;
            }
            SeekFrom::Current(current) => {
                self.seek = self
                    .seek
                    .checked_add_signed(current)
                    .ok_or(FsError::InvalidInput)?;
            }
            SeekFrom::End(end) => {
                self.seek
                    .checked_add_signed(end)
                    .ok_or(FsError::InvalidInput)?;
            }
        }

        Ok(self.seek)
    }

    fn stream_position(&mut self) -> u64 {
        self.seek
    }
}
