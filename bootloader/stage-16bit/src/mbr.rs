use crate::io::{Read, Seek};
use core::{fmt::Debug, marker::PhantomData};

trait ReadSeekCopy: Read + Seek + Copy {}
impl<T: Read + Seek + Copy> ReadSeekCopy for T {}

pub struct Partition<Disk: ReadSeekCopy> {
    pub bootable: bool,
    pub kind: u8,
    pub lba_start: u32,
    pub lba_count: u32,
    seek: u64,
    disk: Disk,
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
#[derive(Clone, Copy)]
pub struct Mbr<Disk: ReadSeekCopy> {
    disk_id: u32,
    reserved: u16,
    entries: [MbrPart; 4],
    signature: u16,
    disk: Disk,
}

impl<Disk: ReadSeekCopy> Mbr<Disk> {
    pub fn new(mut disk: Disk) -> Result<Self, &'static str> {
        let mut sector_buffer = [0u8; 512];
        disk.seek(440);
        disk.read(&mut sector_buffer);

        let mut mbr: Self = unsafe { *sector_buffer.as_ptr().cast() };

        // Its okay to store the disk in here because we immediatly overwrite
        // its sector derived value (the bootloader code) with the disk.
        mbr.disk = disk;

        if mbr.signature != 0xaa55 {
            return Err("Not valid MBR Signature");
        }

        Ok(mbr)
    }

    pub fn partition(&self, index: usize) -> Option<Partition<Disk>> {
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
            disk: self.disk,
        })
    }
}

impl<Disk: ReadSeekCopy> Read for Partition<Disk> {
    fn read(&mut self, buf: &mut [u8]) {
        let seek_offset = self.seek + (self.lba_start as u64 * 512);
        self.disk.seek(seek_offset);

        self.disk.read(buf)
    }
}

impl<Disk: ReadSeekCopy> Seek for Partition<Disk> {
    fn seek(&mut self, pos: u64) -> u64 {
        self.seek = pos;
        self.seek
    }

    fn stream_position(&mut self) -> u64 {
        self.seek
    }
}

impl<Disk: ReadSeekCopy> Debug for Partition<Disk> {
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
