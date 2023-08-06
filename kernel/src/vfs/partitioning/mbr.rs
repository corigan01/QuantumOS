/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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

use crate::vfs::io::{IOResult, PartitionInfo, Read, Seek, SeekFrom, Write};
use crate::vfs::partitioning::PartitionErr;
use crate::vfs::{VFSDisk, VFSDiskID, VFSPartition};
use core::fmt::{Display, Formatter};
use core::mem;
use core::mem::size_of;
use qk_alloc::boxed::Box;
use qk_alloc::vec::Vec;
use quantum_utils::human_bytes::HumanBytes;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct Entry {
    drive_attributes: u8,
    chs_partition_start: u16,
    chs_partition_start_high: u8,
    partition_type: u8,
    chs_partition_end: u16,
    chs_partition_end_high: u8,
    lba_start: u32,
    total_sectors: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct Header {
    bootstrap: [u8; 440],
    disk_id: u32,
    optional: u16,
    entries: [Entry; 4],
    signature: u16,
}

impl Header {
    const READ_ONLY_SIGNATURE: u16 = 0xA5A5;
    const BOOT_SIGNATURE: u16 = 0xAA55;
    const RESERVED_SIGNATURE: u16 = 0x0000;

    pub unsafe fn zeroed() -> Self {
        mem::zeroed()
    }

    pub fn is_valid(&self) -> bool {
        self.signature == Self::BOOT_SIGNATURE || self.signature == Self::READ_ONLY_SIGNATURE
    }
}

pub struct MBR {
    disk_id: VFSDiskID,
    header: Box<Header>,
}

pub struct MBRPartitionEntry {
    disk_id: VFSDiskID,
    seek_current: u64,
    start_lba: u64,
    end_lba: u64,
    partition_type: u8,
    drive_attributes: u8
}

impl MBRPartitionEntry {
    pub fn total_bytes(&self) -> HumanBytes {
        ((self.end_lba - self.start_lba) * 512).into()
    }
}

impl PartitionInfo for MBRPartitionEntry {
    fn seek_start(&self) -> u64 {
        self.start_lba * 512
    }

    fn seek_end(&self) -> u64 {
        self.end_lba * 512
    }

    fn is_bootable(&self) -> bool {
        self.drive_attributes == 0x80
    }
}

impl Seek for MBRPartitionEntry {
    fn seek(&mut self, seek: SeekFrom) -> IOResult<u64> {
        self.seek_current = seek
            .modify_pos(self.total_bytes().into(), 0, self.seek_current)
            .ok_or(PartitionErr::NotSeekable)?;

        // Actually just offset the disk here, so when we go to read/write its just a
        // transparent calling conversion :)
        self.disk_id
            .get_disk_mut()
            .seek(SeekFrom::Start(self.seek_current + self.start_lba))?;

        Ok(self.seek_current)
    }
}

impl Read for MBRPartitionEntry {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize> {
        self.disk_id.get_disk_mut().read(buf)
    }
}

impl Write for MBRPartitionEntry {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        self.disk_id.get_disk_mut().write(buf)
    }

    fn flush(&mut self) -> IOResult<()> {
        self.disk_id.get_disk_mut().flush()
    }
}

impl MBR {
    fn read_header(disk: &mut Box<dyn VFSDisk>) -> IOResult<Box<Header>> {
        disk.seek(SeekFrom::Start(0))?;

        let header = Box::new(unsafe { Header::zeroed() });
        let header_read_buffer = unsafe {
            core::slice::from_raw_parts_mut(header.as_ptr() as *mut u8, size_of::<Header>())
        };

        disk.read_exact(header_read_buffer)?;

        Ok(header)
    }

    pub fn is_mbr_partition(&self) -> bool {
        self.header.is_valid()
    }

    pub fn init(disk: VFSDiskID) -> IOResult<Self> {
        Ok(Self {
            disk_id: disk,
            header: Self::read_header(disk.get_disk_mut())?
        })
    }

    pub fn read_entries(&self) -> IOResult<Vec<Box<dyn VFSPartition>>> {
        let header = &self.header;
        let mut entries: Vec<Box<dyn VFSPartition>> = Vec::new();

        for raw_entry in header.entries {
            let start_lba = raw_entry.lba_start as u64;
            let end_lba = raw_entry.total_sectors as u64 + start_lba;
            let partition_type = raw_entry.partition_type;
            let drive_attributes = raw_entry.drive_attributes;

            if start_lba == 0 && end_lba == 0 {
                continue;
            }

            let mbr_entry = MBRPartitionEntry {
                disk_id: self.disk_id,
                seek_current: 0,
                start_lba,
                end_lba,
                partition_type,
                drive_attributes,
            };

            entries.push(Box::new(mbr_entry));
        }

        Ok(entries)
    }
}

pub fn init_mbr_for_disk(disk_id: VFSDiskID) -> IOResult<()> {
    let mbr = MBR::init(disk_id)?;

    if !mbr.is_mbr_partition() {
        return Err(Box::new(PartitionErr::NotValidPartition));
    }

    let entries = mbr.read_entries()?;
    disk_id.publish_partitions(entries);

    Ok(())
}

impl Display for MBR {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#?}", self.header)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MBRErr {
    CouldNotRead,
    NotValid,
}
