use core::mem::size_of;

#[derive(Clone, Copy)]
pub enum Inode {
    Dir(DirectoryEntry),
    File(DirectoryEntry),
    LongFileName(LongFileName),
}

#[derive(Clone, Copy)]
pub struct DirectoryEntry {
    name: [u8; 11],
    attributes: u8,
    reserved: u8,
    time_tenth: u8,
    creation_time: u16,
    creation_date: u16,
    last_access_date: u16,
    cluster_high: u16,
    modified_time: u16,
    modified_date: u16,
    cluster_low: u16,
    file_size: u32,
}

#[derive(Clone, Copy)]
pub struct LongFileName {
    ordering: u8,
    wchar_low: [u16; 5],
    attributes: u8,
    kind: u8,
    checksum: u8,
    wchar_mid: [u16; 6],
    reserved: u16,
    wchar_high: [u16; 2],
}

impl LongFileName {
    pub fn as_iter<'a>(&'a self) -> LongFileNameIter<'a> {
        LongFileNameIter {
            long_filename: self,
            index: 0,
        }
    }
}

pub struct LongFileNameIter<'a> {
    long_filename: &'a LongFileName,
    index: usize,
}

impl<'a> Iterator for LongFileNameIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.index {
            0..=4 => Some(self.long_filename.wchar_low[self.index as usize] as u8 as char),
            5..=10 => Some(self.long_filename.wchar_mid[self.index as usize - 5] as u8 as char),
            11..=12 => Some(self.long_filename.wchar_high[self.index as usize - 11] as u8 as char),
            _ => None,
        }
    }
}

impl<Array> From<Array> for Inode
where
    Array: AsRef<[u8]>,
{
    fn from(value: Array) -> Self {
        let value = value.as_ref();
        assert!(
            value.len() >= size_of::<Self>(),
            "Byte stream for Inode cannot be less than Inode's size!"
        );

        match value[1] {
            e if e & 0x10 != 0 => Inode::Dir(unsafe { *value.as_ptr().cast::<DirectoryEntry>() }),
            0x0F => Inode::LongFileName(unsafe { *value.as_ptr().cast::<LongFileName>() }),
            _ => Inode::File(unsafe { *value.as_ptr().cast::<DirectoryEntry>() }),
        }
    }
}
