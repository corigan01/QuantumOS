use core::{fmt::Debug, mem::size_of};

use self::bpb::Bpb;
use crate::{
    bios_print, bios_println,
    fatfs::inode::{DirectoryEntry, Inode},
    io::{Read, Seek},
};

mod bpb;
mod inode;

#[derive(Debug)]
pub enum FatKind {
    Fat12,
    Fat16,
    Fat32,
}

pub(super) trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

pub struct Fat<Part: ReadSeek> {
    disk: Part,
    bpb: Bpb,
}

type ClusterId = u32;

enum FatEntry {
    Free,
    Next(ClusterId),
    EOF,
    Reserved,
    Defective,
}

impl FatEntry {
    const FREE_CLUSTER: u32 = 0;
    const ALLOCATED_CLUSTER_BEGIN: u32 = 2;
    const FAT16_MAX: u32 = 0xfff4;
    const FAT16_RESERVED_END: u32 = 0xfff6;
    const FAT16_DEFECTIVE: u32 = Self::FAT16_RESERVED_END + 1;
    const FAT16_EOF: u32 = u16::MAX as u32;
    const FAT32_MAX: u32 = 0xffffff4;
    const FAT32_RESERVED_END: u32 = 0xffffff6;
    const FAT32_DEFECTIVE: u32 = Self::FAT32_RESERVED_END + 1;
    const FAT32_EOF: u32 = u32::MAX;

    fn from_fat16(id: ClusterId) -> FatEntry {
        match id {
            Self::FREE_CLUSTER => FatEntry::Free,
            Self::ALLOCATED_CLUSTER_BEGIN..=Self::FAT16_MAX => FatEntry::Next(id),
            ..=Self::FAT16_RESERVED_END => FatEntry::Reserved,
            Self::FAT16_DEFECTIVE => FatEntry::Defective,
            Self::FAT16_EOF => FatEntry::EOF,
            _ => unreachable!("ClusterID Unknown"),
        }
    }

    fn from_fat32(id: ClusterId) -> FatEntry {
        match id {
            Self::FREE_CLUSTER => FatEntry::Free,
            Self::ALLOCATED_CLUSTER_BEGIN..=Self::FAT32_MAX => FatEntry::Next(id),
            ..=Self::FAT32_RESERVED_END => FatEntry::Reserved,
            Self::FAT32_DEFECTIVE => FatEntry::Defective,
            Self::FAT32_EOF => FatEntry::EOF,
            _ => unreachable!("ClusterID Unknown"),
        }
    }
}

impl<Part: ReadSeek> Fat<Part> {
    pub fn new(mut disk: Part) -> Result<Self, &'static str> {
        let bpb = Bpb::new(&mut disk)?;

        Ok(Self { disk, bpb })
    }

    fn read_fat(&mut self, id: ClusterId) -> Result<FatEntry, &'static str> {
        let fat_region = self.bpb.fat_range();
        let entry_sector = (*fat_region.start()) + (id as u64 / (self.bpb.sector_size() as u64));
        let entry_offset = id as usize % self.bpb.sector_size();

        if entry_sector > *fat_region.end() {
            return Err("Out of range cluster");
        }

        let mut sector_array = [0u8; 512];
        self.disk.seek(entry_sector);
        self.disk.read(&mut sector_array);

        Ok(match self.bpb.kind() {
            FatKind::Fat16 => unsafe {
                FatEntry::from_fat16(
                    (&*core::ptr::slice_from_raw_parts(sector_array.as_ptr() as *const u16, 256))
                        [entry_offset] as ClusterId,
                )
            },
            FatKind::Fat32 => unsafe {
                FatEntry::from_fat32(
                    (&*core::ptr::slice_from_raw_parts(sector_array.as_ptr() as *const u32, 128))
                        [entry_offset] as ClusterId,
                )
            },
            FatKind::Fat12 => todo!("Support reading FAT12"),
        })
    }

    pub fn volume_label<'a>(&'a self) -> &'a str {
        self.bpb.volume_label()
    }

    pub fn print_dir(&mut self, _name: &str) {
        assert_eq!(
            self.bpb.cluster_sectors(),
            2,
            "TODO: Expecting cluster size to be 2 sectors"
        );
        let root_cluster = self.bpb.root_cluster();
        let mut root_data = [0u8; 1024];

        self.disk.seek(self.bpb.cluster_physical_loc(root_cluster));
        self.disk.read(&mut root_data);

        for file_entry in root_data
            .chunks(size_of::<DirectoryEntry>())
            .map(|slice| slice.try_into())
            .filter(|entry: &Result<Inode, _>| entry.is_ok())
        {
            bios_print!("Inode: ");

            match file_entry.unwrap() {
                Inode::LongFileName(_) => bios_print!("LongFileName"),
                Inode::Dir(_) => bios_print!("Dir"),
                Inode::File(_) => bios_print!("File"),
            }

            bios_print!(" ");

            file_entry
                .unwrap()
                .as_iter()
                // .for_each(|c| bios_print!("{:02x}", c as u8));
                .for_each(|c| bios_print!("{}", c));
            bios_println!();
        }

        todo!("Finish printing dir")
    }

    pub fn read(&mut self, name: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
        todo!()
    }
}

impl<Part: ReadSeek> Debug for Fat<Part> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Fat")
            .field("kind", &self.bpb.kind())
            .field("bytes", &(self.bpb.total_sectors() * 512))
            .field("name", &self.volume_label())
            .finish()?;

        Ok(())
    }
}
