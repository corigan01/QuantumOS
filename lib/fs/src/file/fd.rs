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
*
*/

use crate::{
    io::{FileProvider, Metadata, Read, Seek, Write},
    the_vfs, FsResult, Vfs,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FileDescriptor(pub usize);

impl From<usize> for FileDescriptor {
    fn from(value: usize) -> Self {
        FileDescriptor(value)
    }
}

impl Into<usize> for FileDescriptor {
    fn into(self) -> usize {
        self.0
    }
}

impl Read for FileDescriptor {
    fn read(&mut self, buf: &mut [u8]) -> FsResult<usize> {
        the_vfs(|vfs| self.link_vfs(vfs).read(buf))
    }
}

impl Write for FileDescriptor {
    fn write(&mut self, buf: &[u8]) -> FsResult<usize> {
        the_vfs(|vfs| self.link_vfs(vfs).write(buf))
    }

    fn flush(&mut self) -> FsResult<()> {
        the_vfs(|vfs| self.link_vfs(vfs).flush())
    }
}

impl Seek for FileDescriptor {
    fn seek(&mut self, pos: crate::io::SeekFrom) -> FsResult<u64> {
        the_vfs(|vfs| self.link_vfs(vfs).seek(pos))
    }
}

impl Metadata for FileDescriptor {
    fn date_created(&self) -> Option<crate::UnixTime> {
        the_vfs(|vfs| self.link_vfs(vfs).date_created())
    }

    fn date_modified(&self) -> Option<crate::UnixTime> {
        the_vfs(|vfs| self.link_vfs(vfs).date_modified())
    }

    fn date_accessed(&self) -> Option<crate::UnixTime> {
        the_vfs(|vfs| self.link_vfs(vfs).date_accessed())
    }

    fn date_removed(&self) -> Option<crate::UnixTime> {
        the_vfs(|vfs| self.link_vfs(vfs).date_removed())
    }

    fn permissions(&self) -> crate::permission::Permissions {
        the_vfs(|vfs| self.link_vfs(vfs).permissions())
    }

    fn kind(&self) -> crate::io::EntryType {
        the_vfs(|vfs| self.link_vfs(vfs).kind())
    }

    fn can_write(&self) -> bool {
        the_vfs(|vfs| self.link_vfs(vfs).can_write())
    }

    fn can_read(&self) -> bool {
        the_vfs(|vfs| self.link_vfs(vfs).can_read())
    }

    fn can_seek(&self) -> bool {
        the_vfs(|vfs| self.link_vfs(vfs).can_seek())
    }

    fn len(&self) -> u64 {
        the_vfs(|vfs| self.link_vfs(vfs).len())
    }
}

impl FileDescriptor {
    pub fn link_vfs<'a>(self, vfs: &'a mut Vfs) -> VfsLinkedFD<'a> {
        VfsLinkedFD { fd: self, vfs }
    }

    pub fn close(self) -> FsResult<()> {
        the_vfs(|vfs| self.link_vfs(vfs).close())
    }
}

pub struct VfsLinkedFD<'a> {
    fd: FileDescriptor,
    vfs: &'a mut Vfs,
}

impl<'a> VfsLinkedFD<'a> {
    pub fn unlink(self) -> FileDescriptor {
        self.fd
    }

    pub fn close(self) -> FsResult<()> {
        self.vfs.close(self.fd)
    }
}

impl<'a> FileProvider for VfsLinkedFD<'a> {}
impl<'a> Metadata for VfsLinkedFD<'a> {
    fn date_created(&self) -> Option<crate::UnixTime> {
        self.vfs.fd_ref(self.fd).ok()?.data.date_created()
    }

    fn date_modified(&self) -> Option<crate::UnixTime> {
        self.vfs.fd_ref(self.fd).ok()?.data.date_modified()
    }

    fn date_accessed(&self) -> Option<crate::UnixTime> {
        self.vfs.fd_ref(self.fd).ok()?.data.date_accessed()
    }

    fn date_removed(&self) -> Option<crate::UnixTime> {
        self.vfs.fd_ref(self.fd).ok()?.data.date_removed()
    }

    fn permissions(&self) -> crate::permission::Permissions {
        self.vfs.fd_ref(self.fd).unwrap().data.permissions()
    }

    fn kind(&self) -> crate::io::EntryType {
        self.vfs.fd_ref(self.fd).unwrap().data.kind()
    }

    fn can_write(&self) -> bool {
        self.vfs.fd_ref(self.fd).unwrap().data.can_write()
    }

    fn can_read(&self) -> bool {
        self.vfs.fd_ref(self.fd).unwrap().data.can_read()
    }

    fn can_seek(&self) -> bool {
        self.vfs.fd_ref(self.fd).unwrap().data.can_seek()
    }

    fn len(&self) -> u64 {
        self.vfs.fd_ref(self.fd).unwrap().data.len()
    }
}

// TODO: Add the rest of these
impl<'a> Read for VfsLinkedFD<'a> {
    fn read(&mut self, buf: &mut [u8]) -> crate::FsResult<usize> {
        self.vfs.fd_mut(self.fd)?.data.read(buf)
    }
}

impl<'a> Write for VfsLinkedFD<'a> {
    fn write(&mut self, buf: &[u8]) -> crate::FsResult<usize> {
        self.vfs.fd_mut(self.fd)?.data.write(buf)
    }

    fn flush(&mut self) -> crate::FsResult<()> {
        self.vfs.fd_mut(self.fd)?.data.flush()
    }
}

impl<'a> Seek for VfsLinkedFD<'a> {
    fn seek(&mut self, pos: crate::io::SeekFrom) -> crate::FsResult<u64> {
        self.vfs.fd_mut(self.fd)?.data.seek(pos)
    }

    fn rewind(&mut self) -> crate::FsResult<()> {
        self.vfs.fd_mut(self.fd)?.data.rewind()
    }

    fn stream_len(&mut self) -> crate::FsResult<u64> {
        self.vfs.fd_mut(self.fd)?.data.stream_len()
    }

    fn stream_len_dirty(&mut self) -> crate::FsResult<u64> {
        self.vfs.fd_mut(self.fd)?.data.stream_len_dirty()
    }

    fn stream_position(&mut self) -> crate::FsResult<u64> {
        self.vfs.fd_mut(self.fd)?.data.stream_position()
    }
}
