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

use qk_alloc::vec::Vec;

use crate::{
    error::{FsError, FsErrorKind},
    io::FileSystemProvider,
};

mod entry;

pub struct FakeFs {
    nodes: Vec<entry::Entry>,
}

impl FakeFs {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl FileSystemProvider for FakeFs {
    fn supports_permissions(&self) -> bool {
        true
    }

    fn mkdir(
        &mut self,
        path: crate::path::Path,
        permission: crate::permission::Permissions,
    ) -> crate::FsResult<()> {
        Err(FsError::new(
            FsErrorKind::StorageFull,
            "Cannot make directory on FakeFs",
        ))
    }

    fn touch(
        &mut self,
        path: crate::path::Path,
        permission: crate::permission::Permissions,
    ) -> crate::FsResult<()> {
        Err(FsError::new(
            FsErrorKind::StorageFull,
            "Cannot make file on FakeFs",
        ))
    }

    fn rm(&mut self, path: crate::path::Path) -> crate::FsResult<()> {
        Err(FsError::new(
            FsErrorKind::PermissionDenied,
            "Cannot remove files on FakeFs",
        ))
    }

    fn rmdir(&mut self, path: crate::path::Path) -> crate::FsResult<()> {
        Err(FsError::new(
            FsErrorKind::PermissionDenied,
            "Cannot remove files on FakeFs",
        ))
    }

    fn open_file(
        &mut self,
        path: crate::path::Path,
    ) -> crate::FsResult<qk_alloc::boxed::Box<dyn crate::io::FileProvider>> {
        todo!()
    }

    fn open_directory(
        &mut self,
        path: crate::path::Path,
    ) -> crate::FsResult<qk_alloc::boxed::Box<dyn crate::io::DirectoryProvider>> {
        todo!()
    }
}
