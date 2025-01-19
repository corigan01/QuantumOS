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

#![no_std]
#![feature(pointer_is_aligned_to)]

use core::{ffi::FromBytesUntilNulError, str::Utf8Error};
use header::TarHeader;

mod header;

#[derive(Clone, Debug)]
pub enum TarError {
    NotEnoughBytesForHeader,
    BytesNotAligned,
    StrConvertError(FromBytesUntilNulError),
    Utf8Error(Utf8Error),
    NotOctal,
    OutOfRange,
}

#[derive(Clone, Copy)]
pub struct Tar<'a> {
    tar_file: &'a [u8],
}

impl<'a> Tar<'a> {
    pub const fn new(tar_file: &'a [u8]) -> Self {
        Self { tar_file }
    }

    /// Get an iterator of TarFileHeaders to iterate over the tar file
    pub const fn iter(&self) -> TarFileIter<'a> {
        TarFileIter {
            offset: 0,
            tar: *self,
        }
    }
}

pub struct TarFileIter<'a> {
    offset: usize,
    tar: Tar<'a>,
}

impl<'a> Iterator for TarFileIter<'a> {
    type Item = TarFile<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let tar_header = <&TarHeader as TryFrom<_>>::try_from(
            self.tar.tar_file.get(self.offset..self.offset + 512)?,
        )
        .ok()?;

        if tar_header.is_empty() {
            return None;
        }

        let offset = self.offset;
        self.offset += ((tar_header.filesize().ok()? + size_of::<TarHeader>()) / 512 + 1) * 512;

        Some(TarFile {
            tar: self.tar,
            offset,
            header: tar_header,
        })
    }
}

pub struct TarFile<'a> {
    tar: Tar<'a>,
    offset: usize,
    header: &'a header::TarHeader,
}

impl<'a> TarFile<'a> {
    /// Check if this header is empty.
    pub fn is_empty(&self) -> bool {
        self.header.is_empty()
    }

    pub fn filename(&self) -> Result<&str, TarError> {
        self.header.filename()
    }

    /// Checks if this file is of the same name as `compare_name`
    pub fn is_file(&self, compare_name: &str) -> bool {
        self.header.is_file(compare_name)
    }

    /// Attempt to get the filesize.
    pub fn filesize(&self) -> Result<usize, TarError> {
        self.header.filesize()
    }

    /// Get the inner file contents
    pub fn file(&self) -> Result<&'a [u8], TarError> {
        let file_offset = self.offset + 512;
        let file_size = self.filesize()?;

        self.tar
            .tar_file
            .get(file_offset..file_offset + file_size)
            .ok_or(TarError::OutOfRange)
    }
}
