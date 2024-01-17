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
    io::{DirectoryProvider, Metadata},
    path::Path,
    FsResult, Vfs,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DirDescriptor(pub usize);

impl From<usize> for DirDescriptor {
    fn from(value: usize) -> Self {
        DirDescriptor(value)
    }
}

impl Into<usize> for DirDescriptor {
    fn into(self) -> usize {
        self.0
    }
}

impl DirDescriptor {
    pub fn link_vfs<'a>(self, vfs: &'a mut Vfs) -> VfsLinkedDD<'a> {
        VfsLinkedDD { dd: self, vfs }
    }
}

pub struct VfsLinkedDD<'a> {
    dd: DirDescriptor,
    vfs: &'a mut Vfs,
}

impl<'a> VfsLinkedDD<'a> {
    pub fn unlink(self) -> DirDescriptor {
        self.dd
    }

    pub fn close(self) -> FsResult<()> {
        self.vfs.close_dir(self.dd)
    }
}

impl<'a> DirectoryProvider for VfsLinkedDD<'a> {}
impl<'a> Iterator for VfsLinkedDD<'a> {
    type Item = Path;

    fn next(&mut self) -> Option<Self::Item> {
        self.vfs.dd_mut(self.dd).ok()?.data.next()
    }
}

impl<'a> Metadata for VfsLinkedDD<'a> {
    fn date_created(&self) -> Option<crate::UnixTime> {
        self.vfs.dd_ref(self.dd).ok()?.data.date_created()
    }

    fn date_modified(&self) -> Option<crate::UnixTime> {
        self.vfs.dd_ref(self.dd).ok()?.data.date_modified()
    }

    fn date_accessed(&self) -> Option<crate::UnixTime> {
        self.vfs.dd_ref(self.dd).ok()?.data.date_accessed()
    }

    fn date_removed(&self) -> Option<crate::UnixTime> {
        self.vfs.dd_ref(self.dd).ok()?.data.date_removed()
    }

    fn permissions(&self) -> crate::permission::Permissions {
        self.vfs.dd_ref(self.dd).unwrap().data.permissions()
    }

    fn kind(&self) -> crate::io::EntryType {
        self.vfs.dd_ref(self.dd).unwrap().data.kind()
    }

    fn can_write(&self) -> bool {
        self.vfs.dd_ref(self.dd).unwrap().data.can_write()
    }

    fn can_read(&self) -> bool {
        self.vfs.dd_ref(self.dd).unwrap().data.can_read()
    }

    fn can_seek(&self) -> bool {
        self.vfs.dd_ref(self.dd).unwrap().data.can_seek()
    }

    fn len(&self) -> u64 {
        self.vfs.dd_ref(self.dd).unwrap().data.len()
    }
}
