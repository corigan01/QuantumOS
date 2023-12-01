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

use core::ptr::NonNull;

use crate::{
    impl_seek,
    io::{FileProvider, Metadata, Read, Seek, Write},
    path::Path,
    permission::Permissions,
};
use qk_alloc::{boxed::Box, vec::Vec};

pub struct TmpFile {
    pub(crate) count_open: usize,
    pub(crate) path: Path,
    pub(crate) perm: Permissions,
    pub(crate) file_contents: Vec<u8>,
    pub(crate) seek: u64,
}

impl TmpFile {
    pub fn new(path: Path, perm: Permissions) -> Self {
        Self {
            count_open: 0,
            path,
            perm,
            file_contents: Vec::new(),
            seek: 0,
        }
    }

    // For seek impl
    fn seek_current(&self) -> u64 {
        self.seek
    }

    // For seek impl
    fn seek_max(&self) -> u64 {
        self.file_contents.len() as u64
    }

    // For seek impl
    fn set_seek(&mut self, seek: u64) {
        self.seek = seek;
    }
}

impl Read for TmpFile {
    fn read(&mut self, buf: &mut [u8]) -> crate::FsResult<usize> {
        let max_buffer_top =
            core::cmp::min(self.file_contents.len(), buf.len() + self.seek as usize);
        let reading_size = max_buffer_top - self.seek as usize;
        let slice_of_self = &self.file_contents.as_slice()[self.seek as usize..max_buffer_top];

        buf[..(max_buffer_top - self.seek as usize)].copy_from_slice(&slice_of_self);
        self.seek += reading_size as u64;

        Ok(reading_size)
    }
}
impl Write for TmpFile {
    fn write(&mut self, buf: &[u8]) -> crate::FsResult<usize> {
        for byte in buf {
            self.file_contents.push(*byte);
        }
        self.seek += buf.len() as u64;

        Ok(buf.len())
    }

    fn flush(&mut self) -> crate::FsResult<()> {
        Ok(())
    }
}

impl_seek!(TmpFile);

pub struct TmpOpenFile {
    pub(crate) file: NonNull<TmpFile>,
}

impl TmpOpenFile {
    pub fn get_ref(&self) -> &TmpFile {
        let inner = unsafe { self.file.as_ref() };

        assert_ne!(
            inner.count_open, 0,
            "Cannot have a file with an open_count of zero because we are holding an entry!"
        );
        inner
    }

    pub fn get_mut(&mut self) -> &mut TmpFile {
        let inner = unsafe { self.file.as_mut() };

        assert_ne!(
            inner.count_open, 0,
            "Cannot have a file with an open_count of zero because we are holding an entry!"
        );
        inner
    }
}

impl From<&mut Box<TmpFile>> for TmpOpenFile {
    fn from(value: &mut Box<TmpFile>) -> Self {
        value.count_open += 1;

        let Some(ptr) = NonNull::new(value.as_ptr()) else {
            unreachable!("Cannot have a box with a null ptr!");
        };

        Self { file: ptr }
    }
}

impl FileProvider for TmpOpenFile {}
impl Read for TmpOpenFile {
    fn read(&mut self, buf: &mut [u8]) -> crate::FsResult<usize> {
        self.get_mut().read(buf)
    }
}

impl Write for TmpOpenFile {
    fn write(&mut self, buf: &[u8]) -> crate::FsResult<usize> {
        self.get_mut().write(buf)
    }

    fn flush(&mut self) -> crate::FsResult<()> {
        Ok(())
    }
}

impl Seek for TmpOpenFile {
    fn seek(&mut self, pos: crate::io::SeekFrom) -> crate::FsResult<u64> {
        self.get_mut().seek(pos)
    }
}

impl Metadata for TmpOpenFile {
    fn permissions(&self) -> Permissions {
        self.get_ref().perm
    }

    fn can_write(&self) -> bool {
        true
    }

    fn can_read(&self) -> bool {
        true
    }

    fn can_seek(&self) -> bool {
        true
    }

    fn kind(&self) -> crate::io::EntryType {
        crate::io::EntryType::File
    }

    fn len(&self) -> u64 {
        self.get_ref().file_contents.len() as u64
    }
}
