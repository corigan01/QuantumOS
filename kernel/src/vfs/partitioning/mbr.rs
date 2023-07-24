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

use core::fmt::{Display, Formatter};
use qk_alloc::boxed::Box;
use crate::filesystem::impl_disk::{MediumBox, SeekFrom};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct Entry {
    drive_attributes: u8,
    chs_start: [u8; 3],
    partition_type: u8,
    chs_end: [u8; 3],
    lba_start: u32,
    lba_end: u32
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct Header {
    bootstrap: [u8; 440],
    disk_id: u32,
    optional: u16,
    entries: [Entry; 4],
    signature: u16
}

impl Header {
    const READ_ONLY_SIGNATURE: u16 = 0x5A5A;
    const BOOT_SIGNATURE: u16 = 0x55AA;
    const RESERVED_SIGNATURE: u16 = 0x0000;

    pub fn is_valid(&self) -> bool {
        self.signature == Self::BOOT_SIGNATURE || self.signature == Self::READ_ONLY_SIGNATURE
    }
}

pub struct MBR {
    header: Box<Header>
}

impl MBR {

}

impl Display for MBR {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#?}", self.header)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MBRErr {
    CouldNotRead,
    NotValid
}

impl TryFrom<&mut MediumBox> for MBR {
    type Error = MBRErr;

    fn try_from(value: &mut MediumBox) -> Result<Self, Self::Error> {
        value.seek(SeekFrom::Start(0));
        let first_sector = value.read_exact(512).map_err(|_| MBRErr::CouldNotRead)?;

        let read_from_sectors = unsafe { &*(first_sector.as_ptr() as *const Header) };
        if !read_from_sectors.is_valid() {
            return Err(MBRErr::NotValid)
        }

        let box_copy = Box::new(read_from_sectors.clone());

        Ok(Self {
            header: box_copy
        })
    }
}
