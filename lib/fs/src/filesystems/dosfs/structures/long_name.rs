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

use core::mem::size_of;
use core::ptr;
use qk_alloc::string::String;
use crate::error::FsError;
use crate::filesystems::dosfs::structures::{Byte, Word};

// Long name entries are stored in reverse order
#[repr(C, packed)]
pub struct LongNameEntry {
    order: Byte,
    name1: [u16; 5],
    attribute: Byte,
    entry_type: Byte,
    checksum: Byte,
    name2: [u16; 6],
    fst_cluster_low: Word,
    name3: [u16; 2]
}

impl LongNameEntry {
    pub fn is_last_entry(&self) -> bool {
        self.order & 0x40 != 0
    }

    pub fn entry_number(&self) -> usize {
        (self.order & (0x40 - 1)) as usize
    }

    pub fn read_name(&self) -> String {
        // 15 is 1/2 the bytes required to store the entry
        let mut builder_string = String::with_capacity(15);

        // this is needed because rust will complain about it being a packed struct,
        // however, rust copies the value anyway so it kinda looks silly :P
        let read_name1 = self.name1;
        let read_name2 = self.name2;
        let read_name3 = self.name3;

        let big_iterator =
            read_name1.into_iter()
                .chain(read_name2.into_iter())
                .chain(read_name3.into_iter());

        for byte in char::decode_utf16(big_iterator) {
            let Ok(byte) = byte else {
                continue
            };

            if byte as u16 == 0x00 || byte as u16 == 0xFFFF {
                continue
            }

            builder_string.push(byte);
        }

        builder_string
    }
}

impl TryFrom<&[u8]> for LongNameEntry {
    type Error = FsError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < size_of::<Self>() {
            return Err(FsError::try_from_array_error::<Self>(value));
        }

        Ok(unsafe { ptr::read(value.as_ptr() as *const Self) })
    }
}

#[cfg(test)]
mod test {
    use crate::filesystems::dosfs::structures::long_name::LongNameEntry;
    use crate::set_example_allocator;

    #[test]
    fn test_read_name_from_long_name() {
        set_example_allocator(4096);
        let example: [u8; 0x20] = [
            0x41, 0x6B, 0x00, 0x65, 0x00, 0x72, 0x00, 0x6E, 0x00, 0x65, 0x00, 0x0F, 0x00, 0x95, 0x6C, 0x00,
            0x2E, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF
        ];

        let long_name = LongNameEntry::try_from(example.as_ref()).unwrap();

        assert_eq!(long_name.read_name(), "kernel.elf");
        assert_eq!(long_name.entry_number(), 1);
        assert_eq!(long_name.is_last_entry(), true);
    }

    #[test]
    fn test_read_name_bootloader() {
        set_example_allocator(4096);
        let example: [u8; 0x20] = [
            0x41, 0x62, 0x00, 0x6F, 0x00, 0x6F, 0x00, 0x74, 0x00, 0x6C, 0x00, 0x0F, 0x00, 0x17, 0x6F, 0x00,
            0x61, 0x00, 0x64, 0x00, 0x65, 0x00, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF
        ];

        let long_name = LongNameEntry::try_from(example.as_ref()).unwrap();

        assert_eq!(long_name.read_name(), "bootloader");
        assert_eq!(long_name.entry_number(), 1);
        assert_eq!(long_name.is_last_entry(), true);
    }
}