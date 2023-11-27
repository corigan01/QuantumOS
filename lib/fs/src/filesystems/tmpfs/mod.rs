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

use self::{dir::TmpDirectory, file::TmpFile};
use crate::{error::FsError, io::FileSystemProvider};
use qk_alloc::{boxed::Box, vec::Vec};

mod dir;
mod file;

pub struct TmpFs {
    files: Vec<Box<TmpFile>>,
    dires: Vec<Box<TmpDirectory>>,
}

impl TmpFs {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            dires: Vec::new(),
        }
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
        if self.dires.iter().any(|entry| entry.path == path) {
            return Err(FsError::new(
                crate::error::FsErrorKind::AlreadyExists,
                "The directory already exists at that path!",
            ));
        }

        let new_dir = TmpDirectory::new(path, permission);
        self.dires.push(Box::new(new_dir));

        Ok(())
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tmp_dir_new() {
        crate::set_example_allocator();

        let tmpfs = TmpFs::new();
        let mut vfs = crate::Vfs::new();
        assert_eq!(vfs.mount("/".into(), Box::new(tmpfs)), Ok(0));
    }

    #[test]
    fn test_tmp_dir_newfile() {
        crate::set_example_allocator();
    }
}
