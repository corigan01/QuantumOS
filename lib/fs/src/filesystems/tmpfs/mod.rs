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
use qk_alloc::{boxed::Box, vec::Vec};

use crate::{
    io::{DirectoryProvider, FileSystemProvider, Metadata},
    path::Path,
    permission::Permissions,
};

struct TmpFile {
    exists: bool,
    open_count: usize,
    path: Path,
    data: Vec<u8>,
    perm: Permissions,
}

struct TmpAbDir {
    path: Path,
    sub_paths: Vec<Path>,
    perm: Permissions,
}

impl DirectoryProvider for TmpAbDir {}

impl Iterator for TmpAbDir {
    type Item = Path;

    fn next(&mut self) -> Option<Self::Item> {
        self.sub_paths.pop_front()
    }
}

impl Metadata for TmpAbDir {
    fn len(&self) -> u64 {
        0
    }

    fn kind(&self) -> crate::io::EntryType {
        crate::io::EntryType::Directory
    }

    fn permissions(&self) -> Permissions {
        self.perm.clone()
    }

    fn can_read(&self) -> bool {
        true
    }

    fn can_seek(&self) -> bool {
        false
    }

    fn can_write(&self) -> bool {
        true
    }
}

struct TmpAbFile {
    path: Path,
    data: NonNull<TmpFile>,
}

pub struct TmpFs {
    files: Vec<Box<TmpFile>>,
}

impl TmpFs {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }
}

impl FileSystemProvider for TmpFs {
    fn open_directory(
        &mut self,
        path: crate::path::Path,
    ) -> crate::FsResult<qk_alloc::boxed::Box<dyn crate::io::DirectoryProvider>> {
        todo!()
    }

    fn open_file(
        &mut self,
        path: crate::path::Path,
    ) -> crate::FsResult<qk_alloc::boxed::Box<dyn crate::io::FileProvider>> {
        todo!()
    }

    fn rmdir(&mut self, path: crate::path::Path) -> crate::FsResult<()> {
        todo!()
    }

    fn rm(&mut self, path: crate::path::Path) -> crate::FsResult<()> {
        todo!()
    }

    fn mkdir(
        &mut self,
        path: crate::path::Path,
        permission: crate::permission::Permissions,
    ) -> crate::FsResult<()> {
        todo!()
    }

    fn touch(
        &mut self,
        path: crate::path::Path,
        permission: crate::permission::Permissions,
    ) -> crate::FsResult<()> {
        todo!()
    }

    fn supports_permissions(&self) -> bool {
        true
    }
}
