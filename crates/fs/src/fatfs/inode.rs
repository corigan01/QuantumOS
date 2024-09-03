use super::ClusterId;
use crate::error::FsError;
use core::mem::size_of;

#[derive(Clone, Copy, Debug)]
pub enum Inode {
    Dir(DirectoryEntry),
    File(DirectoryEntry),
    LongFileName(LongFileName),
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct DirectoryEntry {
    pub(super) name: [u8; 11],
    pub(super) attributes: u8,
    pub(super) reserved: u8,
    pub(super) time_tenth: u8,
    pub(super) creation_time: u16,
    pub(super) creation_date: u16,
    pub(super) last_access_date: u16,
    pub(super) cluster_high: u16,
    pub(super) modified_time: u16,
    pub(super) modified_date: u16,
    pub(super) cluster_low: u16,
    pub(super) file_size: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct LongFileName {
    pub(super) ordering: u8,
    pub(super) wchar_low: [u16; 5],
    pub(super) attributes: u8,
    pub(super) kind: u8,
    pub(super) checksum: u8,
    pub(super) wchar_mid: [u16; 6],
    pub(super) reserved: u16,
    pub(super) wchar_high: [u16; 2],
}

impl Inode {
    pub fn name_iter<'a>(&'a self) -> NameIter<'a> {
        NameIter {
            entry: self,
            index: 0,
        }
    }
}

pub struct NameIter<'a> {
    entry: &'a Inode,
    index: usize,
}

impl<'a> Iterator for NameIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let return_value = match self.entry {
            Inode::LongFileName(long_name) if (0..=4).contains(&self.index) => {
                Some(long_name.wchar_low[self.index as usize] as u8 as char)
            }
            Inode::LongFileName(long_name) if (5..=10).contains(&self.index) => {
                Some(long_name.wchar_mid[self.index as usize - 5] as u8 as char)
            }
            Inode::LongFileName(long_name) if (11..=12).contains(&self.index) => {
                Some(long_name.wchar_high[self.index as usize - 11] as u8 as char)
            }
            Inode::Dir(dir) if (0..=10).contains(&self.index) => Some(dir.name[self.index] as char),
            Inode::File(file) if (0..=10).contains(&self.index) => {
                Some(file.name[self.index] as char)
            }
            _ => None,
        };

        self.index += 1;

        return_value
    }
}

impl<'a> TryFrom<&'a [u8]> for Inode {
    type Error = FsError;
    fn try_from(value: &'a [u8]) -> Result<Inode, Self::Error> {
        let value = value.as_ref();
        assert!(
            value.len() >= size_of::<DirectoryEntry>(),
            "Byte stream for Inode cannot be less than Inode's size! buf.len() = {}, while size_of::<DirectoryEntry> = {}", value.len(), size_of::<DirectoryEntry>()
        );

        if value.iter().all(|&item| item == 0) {
            return Err(FsError::NotFound);
        }

        match value[11] {
            e if e & 0x10 != 0 => Ok(Inode::Dir(unsafe {
                *value.as_ptr().cast::<DirectoryEntry>()
            })),
            0x0F => Ok(Inode::LongFileName(unsafe {
                *value.as_ptr().cast::<LongFileName>()
            })),
            _ => Ok(Inode::File(unsafe {
                *value.as_ptr().cast::<DirectoryEntry>()
            })),
        }
    }
}

impl DirectoryEntry {
    pub fn cluster_id(&self) -> ClusterId {
        self.cluster_low as u32 | ((self.cluster_high as u32) << 16)
    }
}
