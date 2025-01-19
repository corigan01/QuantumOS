/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

use crate::TarError;

#[repr(C)]
#[derive(Debug)]
pub struct TarHeader {
    filename: [u8; 100],
    mode: [u8; 8],
    uid: [u8; 8],
    gid: [u8; 8],
    size: [u8; 12],
    mtime: [u8; 12],
    checksum: [u8; 8],
    typeflag: [u8; 1],
    reserved: [u8; 355],
}

impl<'a> TryFrom<&'a [u8]> for &'a TarHeader {
    type Error = TarError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() < size_of::<TarHeader>() {
            return Err(TarError::NotEnoughBytesForHeader);
        }

        if !value.as_ptr().is_aligned_to(align_of::<TarHeader>()) {
            return Err(TarError::BytesNotAligned);
        }

        Ok(unsafe { &*value.as_ptr().cast() })
    }
}

impl TarHeader {
    /// Check if this header is empty.
    pub fn is_empty(&self) -> bool {
        self.filename.iter().copied().max().unwrap_or(0) == 0
    }

    pub fn filename(&self) -> Result<&str, TarError> {
        core::ffi::CStr::from_bytes_until_nul(&self.filename)
            .map_err(|convert_error| TarError::StrConvertError(convert_error))
            .and_then(|c_str| {
                c_str
                    .to_str()
                    .map_err(|convert_error| TarError::Utf8Error(convert_error))
            })
    }

    /// Checks if this file is of the same name as `compare_name`
    pub fn is_file(&self, compare_name: &str) -> bool {
        if compare_name.len() > self.filename.len() {
            return false;
        }

        self.filename()
            .is_ok_and(|filename| filename == compare_name)
    }

    /// Attempt to get the filesize.
    pub fn filesize(&self) -> Result<usize, TarError> {
        let mut size = 0;

        for (i, octal_byte) in self
            .size
            .iter()
            .rev()
            .copied()
            .filter(|b| *b != 0)
            .enumerate()
        {
            if octal_byte > b'8' || octal_byte < b'0' {
                return Err(TarError::NotOctal);
            }

            size += 8_usize.pow(i as u32) * ((octal_byte - b'0') as usize);
        }

        Ok(size)
    }
}
