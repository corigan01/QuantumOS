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
    io::{DirectoryProvider, Metadata},
    path::Path,
    permission::Permissions,
};

pub struct TmpDirectory {
    pub(crate) path: Path,
    pub(crate) perm: Permissions,
    pub(crate) entries: Vec<Path>,
}

impl TmpDirectory {
    pub fn new(path: Path, perm: Permissions) -> Self {
        Self {
            path,
            perm,
            entries: Vec::new(),
        }
    }
}

impl DirectoryProvider for TmpDirectory {}
impl Iterator for TmpDirectory {
    type Item = Path;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl Metadata for TmpDirectory {
    fn kind(&self) -> crate::io::EntryType {
        crate::io::EntryType::Directory
    }

    fn permissions(&self) -> Permissions {
        self.perm
    }

    fn can_write(&self) -> bool {
        false
    }

    fn can_read(&self) -> bool {
        false
    }

    fn can_seek(&self) -> bool {
        false
    }

    fn len(&self) -> u64 {
        self.entries.len() as u64
    }
}