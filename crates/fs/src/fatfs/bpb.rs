/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
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

use super::{ClusterId, FatKind, ReadSeek};
use crate::error::{FsError, Result};
use crate::io::SeekFrom;
use core::ops::RangeInclusive;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Bpb {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    number_fats: u8,
    root_entries: u16,
    total_sectors_fat16: u16,
    media_type: u8,
    fat_sectors_fat16: u16,
    sectors_per_track: u16,
    head_count: u16,
    hidden_sectors: u32,
    total_sectors_fat32: u32,
    extended: ExtendedBpb,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Bpb16 {
    drive_number: u8,
    reserved: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_str: [u8; 8],
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Bpb32 {
    fat_size: u32,
    ext_flags: u16,
    fat_version: u32,
    root_cluster: u32,
    fs_info: u16,
    boot_sector: u16,
    reserved: [u8; 12],
    drive_number: u8,
    reserved2: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_str: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
union ExtendedBpb {
    fat16: Bpb16,
    fat32: Bpb32,
}

enum ExtendedKind<'a> {
    Fat16(&'a Bpb16),
    Fat32(&'a Bpb32),
}

impl Bpb {
    const ROOT_ENTRY_SIZE: usize = 32;
    // FIXME: Maybe rust bug? These constants are being used!
    #[allow(unused)]
    const FAT12_CLUSTERS: usize = 4085;
    #[allow(unused)]
    const FAT16_CLUSTERS: usize = 65525;

    pub(crate) fn new<Disk: ReadSeek>(disk: &mut Disk) -> Result<Self> {
        let mut sector_buffer = [0u8; 512];

        disk.seek(SeekFrom::Start(0))?;
        disk.read(&mut sector_buffer)?;

        let bpb: Self = unsafe { *sector_buffer.as_ptr().cast() };

        // TODO: Add more checks for BPB to ensure that it is valid before returning it
        if bpb.bytes_per_sector == 0 || bpb.sectors_per_cluster == 0 || bpb.jmp_boot[0] != 0xEB {
            return Err(FsError::InvalidInput);
        }

        Ok(bpb)
    }

    pub fn sector_size(&self) -> usize {
        self.bytes_per_sector as usize
    }

    fn root_sectors(&self) -> usize {
        // 3.5 Determination of FAT type when mounting the Volume (page: 14)
        ((self.root_entries as usize * Self::ROOT_ENTRY_SIZE)
            + (self.bytes_per_sector as usize - 1))
            / (self.bytes_per_sector as usize)
    }

    pub fn total_sectors(&self) -> usize {
        if self.total_sectors_fat16 != 0 {
            self.total_sectors_fat16 as usize
        } else {
            self.total_sectors_fat32 as usize
        }
    }

    fn fat_sectors(&self) -> usize {
        if self.fat_sectors_fat16 != 0 {
            self.fat_sectors_fat16 as usize
        } else {
            unsafe { self.extended.fat32.fat_size as usize }
        }
    }

    fn clusters(&self) -> usize {
        let data_sectors = self.total_sectors()
            - (self.reserved_sectors as usize
                + (self.number_fats as usize * self.fat_sectors())
                + self.root_sectors());

        data_sectors / (self.sectors_per_cluster as usize)
    }

    pub fn kind(&self) -> FatKind {
        match self.clusters() {
            ..=Self::FAT12_CLUSTERS => FatKind::Fat12,
            ..=Self::FAT16_CLUSTERS => FatKind::Fat16,
            _ => FatKind::Fat32,
        }
    }

    fn safe_extended<'a>(&'a self) -> ExtendedKind<'a> {
        match self.kind() {
            FatKind::Fat12 | FatKind::Fat16 => ExtendedKind::Fat16(unsafe { &self.extended.fat16 }),
            FatKind::Fat32 => ExtendedKind::Fat32(unsafe { &self.extended.fat32 }),
        }
    }

    pub fn fat_range(&self) -> RangeInclusive<u64> {
        let fat_start = self.reserved_sectors as u64;
        let fat_end = fat_start + (self.fat_sectors() as u64);

        fat_start..=fat_end
    }

    pub fn volume_label<'a>(&'a self) -> &'a str {
        match self.safe_extended() {
            ExtendedKind::Fat16(ext) => core::str::from_utf8(&ext.volume_label).unwrap(),
            ExtendedKind::Fat32(ext) => core::str::from_utf8(&ext.volume_label).unwrap(),
        }
    }

    pub fn root_cluster(&self) -> ClusterId {
        match self.safe_extended() {
            ExtendedKind::Fat16(_) => 0,
            ExtendedKind::Fat32(ext) => ext.root_cluster as ClusterId,
        }
    }

    pub fn cluster_physical_loc(&self, cluster: ClusterId) -> u64 {
        let common =
            self.reserved_sectors as u64 + (self.fat_sectors() as u64 * self.number_fats as u64);
        if cluster == 0 {
            return common * (self.bytes_per_sector as u64);
        }

        let first_data_sector = common + self.root_sectors() as u64;
        let cluster_sectors = self.sectors_per_cluster as u64;
        (first_data_sector + ((cluster - 2) as u64 * cluster_sectors))
            * (self.bytes_per_sector as u64)
    }

    pub fn cluster_sectors(&self) -> usize {
        self.sectors_per_cluster as usize
    }

    pub fn fat_entry_bytes(&self) -> usize {
        match self.kind() {
            FatKind::Fat12 => todo!("Fat12 not impl"),
            FatKind::Fat16 => 2,
            FatKind::Fat32 => 4,
        }
    }
}
