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

use core::fmt::Debug;
use fs::error::{FsError, Result};
use fs::io::{Read, Seek, SeekFrom};

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

pub struct Partition<'a, Disk: ReadSeek> {
    pub bootable: bool,
    pub kind: u8,
    pub lba_start: u32,
    pub lba_count: u32,
    seek: u64,
    disk: &'a mut Disk,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct MbrPart {
    boot_flag: u8,
    start_chs: [u8; 3],
    kind: u8,
    end_chs: [u8; 3],
    sector_start: u32,
    count: u32,
}

#[repr(C)]
pub struct Mbr<Disk: ReadSeek> {
    disk_id: u32,
    reserved: u16,
    entries: [MbrPart; 4],
    signature: u16,
    disk: Disk,
}

impl<Disk: ReadSeek> Mbr<Disk> {
    pub fn new(mut disk: Disk) -> Result<Self> {
        let mut mbr: Self = unsafe { core::mem::zeroed() };

        disk.seek(SeekFrom::Start(440))?;
        disk.read(unsafe { core::slice::from_raw_parts_mut((&mut mbr as *mut Self).cast(), 512) })?;

        // Its okay to store the disk in here because we immediatly overwrite
        // its sector derived value (the bootloader code) with the disk.
        mbr.disk = disk;

        if mbr.signature != 0xaa55 {
            return Err(FsError::InvalidInput);
        }

        Ok(mbr)
    }

    pub fn partition<'a>(&'a mut self, index: usize) -> Option<Partition<'a, Disk>> {
        let entry = &self.entries.get(index)?;

        if entry.count == 0 || entry.sector_start == 0 {
            return None;
        }

        Some(Partition::<Disk> {
            bootable: entry.boot_flag == 0x80,
            kind: entry.kind,
            lba_start: entry.sector_start,
            lba_count: entry.count,
            seek: 0,
            disk: &mut self.disk,
        })
    }
}

impl<'a, Disk: ReadSeek> Read for Partition<'a, Disk> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let seek_offset = self.seek + (self.lba_start as u64 * 512);
        self.disk.seek(SeekFrom::Start(seek_offset))?;

        self.disk.read(buf)
    }
}

impl<'a, Disk: ReadSeek> Seek for Partition<'a, Disk> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            SeekFrom::Start(pos) => self.seek = pos,
            _ => todo!("Seek is not fully implemented"),
        }

        Ok(self.seek)
    }

    fn stream_position(&mut self) -> u64 {
        self.seek
    }
}

impl<'a, Disk: ReadSeek> Debug for Partition<'a, Disk> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Partition")
            .field("bootable", &self.bootable)
            .field("kind", &format_args!("0x{:02x}", &self.kind))
            .field("lba_start", &self.lba_start)
            .field("lba_count", &self.lba_count)
            .finish()?;

        Ok(())
    }
}
