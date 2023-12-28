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

use crate::error::FsError;
use qk_alloc::vec::Vec;

pub struct MbrPartition {
    lba_start: usize,
    lba_end: usize,
    bootable: bool,
}

pub struct Mbr {
    disk_id: u32,
    partitions: Vec<MbrPartition>,
    bootable: bool,
}

#[repr(C, packed)]
struct RawMbrEntry {
    attributes: u8,
    chs_start: [u8; 3],
    partition_type: u8,
    chs_end: [u8; 3],
    lba_start: u32,
    lba_count: u32,
}

#[repr(C, packed)]
struct RawMbr {
    bootstrap: [u8; 440],
    disk_id: u32,
    reserved: u16,
    entries: [RawMbrEntry; 4],
    boot_signature: u16,
}

impl TryFrom<&[u8]> for Mbr {
    type Error = FsError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 512 {
            return Err(FsError::try_from_array_error::<RawMbrEntry>(value));
        }

        let raw_mbr = unsafe { &*(value.as_ptr() as *const RawMbr) };

        if raw_mbr.boot_signature != 0xAA55 {
            return Err(FsError::new(
                crate::error::FsErrorKind::InvalidData,
                "MBR Partition does not contain 0xAA55",
            ));
        }

        let mut partitions = Vec::new();
        for entry in raw_mbr.entries.iter() {
            let lba_start = entry.lba_start as usize;
            let lba_count = entry.lba_count as usize;
            let lba_end = lba_start + lba_count;
            let att = entry.attributes;

            partitions.push(MbrPartition {
                lba_start,
                lba_end,
                bootable: att & (1 << 7) != 0,
            });
        }

        Ok(Mbr {
            disk_id: raw_mbr.disk_id,
            partitions,
            bootable: true,
        })
    }
}
