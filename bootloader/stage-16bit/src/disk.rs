use bios::disk::raw_read;
use bios::BiosStatus;
use fs::read_block::BlockDevice;

use fs::error::{FsError, Result};
use fs::io::{Read, Seek, SeekFrom};

const MAX_SECTORS_PER_READ: usize = 32;

#[link_section = ".buffer"]
static mut TEMP_BUFFER: [u8; 512] = [0u8; 512];

#[derive(Clone, Copy)]
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
        match raw_read(self.id, block_offset, 1, unsafe {
            TEMP_BUFFER.as_mut_ptr()
        }) {
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
