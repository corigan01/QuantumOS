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

use crate::{
    impl_seek,
    io::{FileProvider, Metadata, Read, Write},
    path::Path,
    permission::Permissions,
};
use qk_alloc::vec::Vec;

pub struct TmpFile {
    pub(crate) path: Path,
    pub(crate) perm: Permissions,
    pub(crate) file_contents: Vec<u8>,
    pub(crate) seek: u64,
}

impl TmpFile {
    pub fn new(path: Path, perm: Permissions) -> Self {
        Self {
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

impl FileProvider for TmpFile {}
impl Read for TmpFile {
    fn read(&mut self, buf: &mut [u8]) -> crate::FsResult<usize> {
        let max_buffer_top =
            core::cmp::min(self.file_contents.len(), buf.len() + self.seek as usize);
        let reading_size = max_buffer_top - self.seek as usize;
        let slice_of_self = &self.file_contents.as_slice()[self.seek as usize..max_buffer_top];

        buf.copy_from_slice(&slice_of_self);
        self.seek += reading_size as u64;

        Ok(reading_size)
    }
}
impl Write for TmpFile {
    fn write(&mut self, buf: &[u8]) -> crate::FsResult<usize> {
        let max_buffer_top = core::cmp::min(self.file_contents.len(), buf.len());
        let writting_size = max_buffer_top - self.seek as usize;
        let slice_of_self =
            &mut self.file_contents.as_mut_slice()[self.seek as usize..max_buffer_top];

        slice_of_self.copy_from_slice(&buf[..writting_size]);
        self.seek += writting_size as u64;

        Ok(writting_size)
    }

    fn flush(&mut self) -> crate::FsResult<()> {
        Ok(())
    }
}

impl_seek!(TmpFile);

impl Metadata for TmpFile {
    fn permissions(&self) -> Permissions {
        self.perm
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
        self.file_contents.len() as u64
    }
}
