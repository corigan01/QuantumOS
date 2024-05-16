use bios::disk;
use core::ptr;

use crate::error::Result;
use crate::io::{Read, Seek};

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

impl Seek for BiosDisk {
    fn seek(&mut self, pos: u64) -> u64 {
        self.seek = pos;
        pos
    }

    fn stream_position(&mut self) -> u64 {
        self.seek
    }
}

impl Read for BiosDisk {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut reading_start = self.seek;
        let mut reading_end = reading_start + (buf.len() as u64);

        let mut starting_sector = reading_start / 512;
        let ending_sector = reading_end / 512;

        let mut buf_ptr = buf.as_mut_ptr();

        // Not aligned start
        let non_alignment_start = reading_start % 512;
        if non_alignment_start != 0 {
            disk::raw_read(self.id, starting_sector, 1, unsafe {
                TEMP_BUFFER.as_mut_ptr()
            })
            .unwrap();

            unsafe {
                ptr::copy_nonoverlapping(
                    TEMP_BUFFER.as_ptr().add(non_alignment_start as usize),
                    buf.as_mut_ptr(),
                    (512 - non_alignment_start as usize).min(buf.len()),
                )
            };

            starting_sector += 1;
            reading_start += 512 - non_alignment_start;
            buf_ptr = unsafe { buf_ptr.add(non_alignment_start as usize) };
        }

        // not aligned end
        let non_alignment_end = reading_end & 0x1FF;
        if non_alignment_end != 0 && ending_sector > starting_sector {
            disk::raw_read(self.id, ending_sector + 1, 1, unsafe {
                TEMP_BUFFER.as_mut_ptr().add(non_alignment_end as usize)
            })
            .unwrap();

            unsafe {
                ptr::copy_nonoverlapping(
                    TEMP_BUFFER.as_ptr(),
                    buf.as_mut_ptr()
                        .add((reading_end - non_alignment_end) as usize),
                    non_alignment_end as usize,
                );
            }

            reading_end -= non_alignment_end;
        }

        assert!(
            reading_start % 512 == 0,
            "Reading Start should be aligned: {}",
            reading_start
        );
        assert!(
            reading_end % 512 == 0 || ending_sector <= starting_sector,
            "Reading End should be aligned: {}",
            reading_end
        );

        while starting_sector < ending_sector {
            let sectors_to_read =
                ((ending_sector - starting_sector) as usize).min(MAX_SECTORS_PER_READ);
            disk::raw_read(self.id, starting_sector, sectors_to_read, buf_ptr).unwrap();

            starting_sector += sectors_to_read as u64;
            buf_ptr = unsafe { buf_ptr.add(sectors_to_read as usize * 512) };
        }

        // FIXME: Use real bytes read to use here
        Ok(0)
    }
}
