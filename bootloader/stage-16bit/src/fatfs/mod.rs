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

    pub fn cluster_of(&mut self, name: &str) -> Result<ClusterId, &'static str> {
        assert_eq!(
            self.bpb.cluster_sectors(),
            2,
            "TODO: Expecting cluster size to be 2 sectors"
        );

        let mut path = name.split('/').filter(|str| !str.is_empty()).peekable();
        let mut inode_cluster = self.bpb.root_cluster();
        let mut data = [0u8; 1024];

        'outer: loop {
            let Some(path_part) = path.next() else {
                todo!("path_part is somehow none");
            };

            // Max string size for FAT is 256-chars
            let mut filename_str = [0u8; 256];
            let mut filename_len = 0;

            self.disk.seek(self.bpb.cluster_physical_loc(inode_cluster));
            self.disk.read(&mut data);

            for inode in data
                .chunks(size_of::<DirectoryEntry>())
                .map(|slice| slice.try_into())
                .filter_map(|entry: Result<Inode, _>| entry.ok())
            {
                let filename = core::str::from_utf8(&filename_str[..filename_len])
                    .unwrap_or("")
                    .trim();

                match inode {
                    Inode::LongFileName(lfn) => {
                        let ordering_number = (lfn.ordering - 1) & (u8::MAX ^ 0x40);
                        let offset = (ordering_number * 13) as usize;

                        filename_str[offset..(offset + 13)]
                            .iter_mut()
                            .zip(
                                inode
                                    .name_iter()
                                    .filter(|lfn_c| lfn_c.is_ascii() && *lfn_c != '\0'),
                            )
                            .for_each(|(filename_c, inode_c)| {
                                *filename_c = inode_c as u8;
                                filename_len += 1;
                            });
                    }
                    Inode::Dir(entry) => {
                        bios_println!("DIR : name = '{}' key = '{}'", filename, path_part);
                        if path_part.trim().eq_ignore_ascii_case(filename) {
                            // more todo
                            if path.peek().is_some() {
                                inode_cluster = entry.cluster_id();
                                continue 'outer;
                            }

                            return Ok(entry.cluster_id());
                        }

                        filename_str = [0u8; 256];
                        filename_len = 0;
                        continue;
                    }
                    Inode::File(file) => {
                        bios_println!(
                            "FILE: name = '{}' -- {:?} key = '{}'",
                            filename,
                            core::str::from_utf8(&file.name),
                            path_part
                        );

                        // Files cannot have other files after it in the path:
                        // So, we must not be the one.
                        if path.peek().is_some() {
                            filename_str = [0u8; 256];
                            filename_len = 0;
                            continue;
                        }

                        if path_part.trim().eq_ignore_ascii_case(filename) {
                            return Ok(file.cluster_id());
                        }

                        filename_str = [0u8; 256];
                        filename_len = 0;
                    }
                }
            }

            return Err("Could not find file");
        }
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
