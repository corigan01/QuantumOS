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
    error::{FsError, FsErrorKind},
    fd::FileDescriptor,
    io::{FileProvider, FileSystemProvider},
    path::Path,
    permission::Permissions,
    FsResult,
};
use qk_alloc::{bitfield::Bitmap, boxed::Box, vec::Vec};

pub type FilesystemID = usize;

struct OpenItem {
    id: FileDescriptor,
    fs_id: FilesystemID,
    path: Path,
    data: Box<dyn FileProvider>,
}

struct OpenFs {
    id: FilesystemID,
    path: Path,
    data: Box<dyn FileSystemProvider>,
}

pub struct BitQueue<Type> {
    mask: Bitmap,
    vec: Vec<Option<Type>>,
}

impl<Type> BitQueue<Type> {
    pub fn new() -> Self {
        Self {
            mask: Bitmap::new(),
            vec: Vec::new(),
        }
    }

    pub fn first_free(&mut self) -> usize {
        self.mask.first_of(false).unwrap_or(self.vec.len())
    }

    pub fn queue(&mut self, value: Type) -> usize {
        let first_free = self.first_free();
        if first_free >= self.vec.len() {
            self.vec.push(Some(value));
        } else {
            self.vec[first_free] = Some(value);
        }

        self.mask.set_bit(first_free, true);
        first_free
    }

    pub fn try_remove(&mut self, location: usize) -> Option<Type> {
        if !self.mask.get_bit(location) {
            return None;
        }

        self.mask.set_bit(location, false);
        let value = unsafe { core::ptr::read(&self.vec[location] as *const Option<Type>) };
        self.vec[location] = None;

        value
    }

    pub fn remove(&mut self, location: usize) -> Type {
        self.try_remove(location)
            .expect("cannot remove a location that does not exist!")
    }

    pub fn iter(&self) -> impl Iterator<Item = &Type> {
        self.vec
            .iter()
            .filter(|val| val.is_some())
            .map(|val| val.as_ref().unwrap())
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }
}

impl<Type> core::ops::Index<usize> for BitQueue<Type> {
    type Output = Type;
    fn index(&self, index: usize) -> &Self::Output {
        self.vec[index].as_ref().expect("That index does not exist")
    }
}

impl<Type> core::ops::IndexMut<usize> for BitQueue<Type> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.vec[index].as_mut().expect("that index does not exist")
    }
}

pub struct Vfs {
    open_ids: BitQueue<OpenItem>,
    filesystems: BitQueue<OpenFs>,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            open_ids: BitQueue::new(),
            filesystems: BitQueue::new(),
        }
    }

    pub fn mount(
        &mut self,
        path: Path,
        device: Box<dyn FileSystemProvider>,
    ) -> FsResult<FilesystemID> {
        let id = self.filesystems.first_free();
        self.filesystems.queue(OpenFs {
            id,
            path,
            data: device,
        });

        Ok(id)
    }

    fn files_open_with_fsid(&mut self, fsid: FilesystemID) -> usize {
        self.open_ids
            .iter()
            .filter(|entry| entry.fs_id == fsid)
            .count()
    }

    fn get_provider_for_path(&mut self, path: &Path) -> Option<FilesystemID> {
        todo!()
    }

    pub fn umount(&mut self, path: Path) -> FsResult<Box<dyn FileSystemProvider>> {
        let truncated_path = path.truncate_path();
        let id = self
            .filesystems
            .iter()
            .find_map(|entry| {
                if entry.path == truncated_path {
                    Some(entry.id)
                } else {
                    None
                }
            })
            .ok_or(FsError::new(
                FsErrorKind::NotFound,
                "The filesystem does not exist at that path!",
            ))?;

        self.unmount_id(id)
    }

    pub fn unmount_id(&mut self, id: FilesystemID) -> FsResult<Box<dyn FileSystemProvider>> {
        if id >= self.filesystems.len() {
            return Err(FsError::new(
                FsErrorKind::NotFound,
                "That filesystem id does not exist!",
            ));
        }

        let files_open = self.files_open_with_fsid(id);
        if files_open != 0 {
            return Err(FsError::new(
                FsErrorKind::AddrInUse,
                "That filesystem is currently in-use and cannot be unmounted!",
            ));
        }

        Ok(self.filesystems.remove(id).data)
    }

    pub fn open(&mut self, path: Path) -> FsResult<FileDescriptor> {
        let path = path.truncate_path();

        let fsid = self.get_provider_for_path(&path).ok_or(FsError::new(
            FsErrorKind::NotFound,
            "That files does not exist!",
        ))?;

        let fs = &mut self.filesystems[fsid];
        let fs_mount = fs.path.clone();

        let fs_rel_path = path.clone().snip_off(fs_mount).ok_or(FsError::new(
            FsErrorKind::InvalidData,
            "path cannot snip to relative path for sub-filesystem",
        ))?;

        let file_child = fs.data.open_file(fs_rel_path)?;
        let file_id = self.open_ids.first_free().into();
        self.open_ids.queue(OpenItem {
            id: file_id,
            fs_id: fsid,
            path,
            data: file_child,
        });

        Ok(file_id)
    }

    pub fn close(&mut self, fd: FileDescriptor) -> FsResult<()> {
        todo!()
    }

    pub fn touch(&mut self, path: Path, perm: Permissions) -> FsResult<()> {
        todo!()
    }

    pub fn rm(&mut self, path: Path) -> FsResult<()> {
        todo!()
    }

    pub fn mkdir(&mut self, path: Path, perm: Permissions) -> FsResult<()> {
        todo!()
    }

    pub fn rmdir(&mut self, path: Path) -> FsResult<()> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bitqueue_queue() {
        crate::set_example_allocator();

        let mut bq = BitQueue::new();

        bq.queue(0);

        assert_eq!(bq.len(), 1);
        assert_eq!(bq.first_free(), 1);
    }

    #[test]
    fn test_bitqueue_remove() {
        crate::set_example_allocator();

        let mut bq = BitQueue::new();

        for i in 0..100 {
            bq.queue(i);
        }

        assert_eq!(bq.len(), 100);
        assert_eq!(bq.first_free(), 100);

        bq.remove(10);
        bq.remove(20);
        bq.remove(31);

        assert_eq!(bq.len(), 97);
        assert_eq!(bq.first_free(), 10);
    }

    #[test]
    fn test_bitqueue_both() {
        crate::set_example_allocator();

        let mut bq = BitQueue::new();

        for i in 0..100 {
            bq.queue(i);
        }

        assert_eq!(bq.len(), 100);
        assert_eq!(bq.first_free(), 100);

        bq.remove(20);
        bq.remove(21);
        bq.remove(90);

        assert_eq!(bq.len(), 97);
        assert_eq!(bq.first_free(), 20);

        assert_eq!(bq.queue(20), 20);
        assert_eq!(bq.len(), 98);
        assert_eq!(bq.first_free(), 21);
        assert_eq!(bq.queue(-1), 21);
        assert_eq!(bq.len(), 99);
        assert_eq!(bq.first_free(), 90);
    }

    #[test]
    fn test_add_and_remove_all() {
        crate::set_example_allocator();

        let mut bq = BitQueue::new();

        for _ in 0..100 {
            for i in 0..100 {
                bq.queue(i);
            }

            assert_eq!(bq.len(), 100);

            for i in 0..100 {
                bq.remove(i);
            }

            assert_eq!(bq.len(), 0);
        }
    }
}
