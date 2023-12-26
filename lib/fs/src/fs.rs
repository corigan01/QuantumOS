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
    dir::dd::DirDescriptor,
    error::{FsError, FsErrorKind},
    fd::FileDescriptor,
    io::{DirectoryProvider, FileProvider, FileSystemProvider, SeekFrom},
    path::Path,
    permission::Permissions,
    FsResult,
};
use qk_alloc::{bitfield::Bitmap, boxed::Box, vec::Vec};

pub type FilesystemID = usize;

pub(crate) struct OpenFile {
    pub id: FileDescriptor,
    pub fs_id: Option<FilesystemID>,
    pub path: Path,
    pub open_count: usize,
    pub data: Box<dyn FileProvider>,
}

pub(crate) struct OpenDir {
    pub id: DirDescriptor,
    pub fs_id: FilesystemID,
    pub path: Path,
    pub open_count: usize,
    pub data: Box<dyn DirectoryProvider>,
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
        unsafe { core::ptr::write((&mut self.vec[location]) as *mut Option<Type>, None) };

        value
    }

    pub fn get_state(&self, location: usize) -> bool {
        self.mask.get_bit(location)
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
    open_ids: BitQueue<OpenFile>,
    open_dds: BitQueue<OpenDir>,
    filesystems: BitQueue<OpenFs>,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            open_ids: BitQueue::new(),
            open_dds: BitQueue::new(),
            filesystems: BitQueue::new(),
        }
    }

    pub fn mount(
        &mut self,
        path: Path,
        device: Box<dyn FileSystemProvider>,
    ) -> FsResult<FilesystemID> {
        if self.filesystems.iter().any(|entry| entry.path == path) {
            return Err(FsError::new(
                FsErrorKind::AlreadyExists,
                "Filesystem already amounted",
            ));
        }

        let id = self.filesystems.first_free();
        self.filesystems.queue(OpenFs {
            id,
            path: path.truncate_path(),
            data: device,
        });

        Ok(id)
    }

    fn files_open_with_fsid(&mut self, fsid: FilesystemID) -> usize {
        self.open_ids
            .iter()
            .filter(|entry| entry.fs_id == Some(fsid))
            .count()
    }

    fn get_provider_for_path(&self, path: &Path) -> Option<FilesystemID> {
        self.filesystems
            .iter()
            .filter_map(|entry| {
                let entry_path = entry.path.as_str();
                let provider_path = path.clone().truncate_path();

                if provider_path.as_str().starts_with(entry_path) {
                    Some((entry.path.as_str().len(), entry.id))
                } else {
                    None
                }
            })
            .max_by_key(|(len, _)| *len)
            .map(|(_, id)| id)
    }

    fn get_fs_and_rel_path(&self, path: Path) -> FsResult<(FilesystemID, Path)> {
        let path = path.truncate_path();

        let fsid = self.get_provider_for_path(&path).ok_or(FsError::new(
            FsErrorKind::NotFound,
            "That files does not exist!",
        ))?;

        let fs = &self.filesystems[fsid];
        let fs_mount = fs.path.clone();

        let mut fs_rel_path = path.remove_parent(&fs_mount).ok_or(FsError::new(
            FsErrorKind::InvalidData,
            "path cannot snip to relative path for sub-filesystem",
        ))?;

        if !fs_rel_path.as_str().starts_with("/") {
            fs_rel_path.prepend("/");
        }

        Ok((fsid, fs_rel_path))
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

        let removed = self.filesystems.remove(id);
        Ok(removed.data)
    }

    pub fn open_custom(
        &mut self,
        path: Path,
        custom: Box<dyn FileProvider>,
    ) -> FsResult<FileDescriptor> {
        let path = path.truncate_path();

        let file_id = self.open_ids.first_free().into();
        self.open_ids.queue(OpenFile {
            id: file_id,
            fs_id: None,
            path,
            open_count: 1,
            data: custom,
        });

        Ok(file_id)
    }

    pub fn open(&mut self, path: Path) -> FsResult<FileDescriptor> {
        let path = path.truncate_path();
        let (fsid, fs_rel_path) = self.get_fs_and_rel_path(path.clone())?;
        let fs = &mut self.filesystems[fsid];

        let file_child = fs.data.open_file(fs_rel_path)?;
        let file_id = self.open_ids.first_free().into();
        self.open_ids.queue(OpenFile {
            id: file_id,
            fs_id: Some(fsid),
            path,
            open_count: 1,
            data: file_child,
        });

        Ok(file_id)
    }

    pub fn open_dir(&mut self, path: Path) -> FsResult<DirDescriptor> {
        let path = path.truncate_path();

        let (fsid, fs_rel_path) = self.get_fs_and_rel_path(path.clone())?;
        let fs = &mut self.filesystems[fsid];

        let dir_child = fs.data.open_directory(fs_rel_path)?;
        let dir_id = self.open_dds.first_free().into();
        self.open_dds.queue(OpenDir {
            id: dir_id,
            fs_id: fsid,
            path,
            open_count: 1,
            data: dir_child,
        });

        Ok(dir_id)
    }

    pub fn close(&mut self, fd: FileDescriptor) -> FsResult<()> {
        if !self.open_ids.get_state(fd.0) {
            return Err(FsError::new(
                FsErrorKind::InvalidInput,
                "That fd does not exist!",
            ));
        }

        // If we have more then 1 of these open, we just decrease the ref count
        if self.open_ids[fd.0].open_count > 1 {
            self.open_ids[fd.0].open_count -= 1;
            return Ok(());
        }

        self.open_ids.remove(fd.0);
        Ok(())
    }

    pub fn close_dir(&mut self, dd: DirDescriptor) -> FsResult<()> {
        if !self.open_dds.get_state(dd.0) {
            return Err(FsError::new(
                FsErrorKind::InvalidInput,
                "That dd does not exist!",
            ));
        }

        // If we have more then 1 of these open, we just decrease the ref count
        if self.open_dds[dd.0].open_count > 1 {
            self.open_dds[dd.0].open_count -= 1;
            return Ok(());
        }

        self.open_dds.remove(dd.0);
        Ok(())
    }

    pub fn touch(&mut self, path: Path, perm: Permissions) -> FsResult<()> {
        let (fsid, fs_rel_path) = self.get_fs_and_rel_path(path)?;
        let fs = &mut self.filesystems[fsid];

        fs.data.touch(fs_rel_path, perm)
    }

    pub fn rm(&mut self, path: Path) -> FsResult<()> {
        let (fsid, fs_rel_path) = self.get_fs_and_rel_path(path)?;
        let fs = &mut self.filesystems[fsid];

        fs.data.rm(fs_rel_path)
    }

    pub fn mkdir(&mut self, path: Path, perm: Permissions) -> FsResult<()> {
        let (fsid, fs_rel_path) = self.get_fs_and_rel_path(path)?;
        let fs = &mut self.filesystems[fsid];

        fs.data.mkdir(fs_rel_path, perm)
    }

    pub fn rmdir(&mut self, path: Path) -> FsResult<()> {
        let (fsid, fs_rel_path) = self.get_fs_and_rel_path(path)?;
        let fs = &mut self.filesystems[fsid];

        fs.data.rmdir(fs_rel_path)
    }

    pub(crate) fn fd_ref(&self, fd: FileDescriptor) -> FsResult<&OpenFile> {
        if self.open_ids.len() < fd.0 {
            return Err(FsError::new(
                FsErrorKind::NotFound,
                "That file descriptor was not found!",
            ));
        }

        Ok(&self.open_ids[fd.0])
    }

    pub(crate) fn fd_mut(&mut self, fd: FileDescriptor) -> FsResult<&mut OpenFile> {
        if self.open_ids.len() < fd.0 {
            return Err(FsError::new(
                FsErrorKind::NotFound,
                "That file descriptor was not found!",
            ));
        }

        Ok(&mut self.open_ids[fd.0])
    }
    pub(crate) fn dd_ref(&self, dd: DirDescriptor) -> FsResult<&OpenDir> {
        if self.open_dds.len() < dd.0 {
            return Err(FsError::new(
                FsErrorKind::NotFound,
                "That file descriptor was not found!",
            ));
        }

        Ok(&self.open_dds[dd.0])
    }

    pub(crate) fn dd_mut(&mut self, dd: DirDescriptor) -> FsResult<&mut OpenDir> {
        if self.open_dds.len() < dd.0 {
            return Err(FsError::new(
                FsErrorKind::NotFound,
                "That file descriptor was not found!",
            ));
        }

        Ok(&mut self.open_dds[dd.0])
    }

    pub fn fread(&mut self, fd: FileDescriptor, buf: &mut [u8]) -> FsResult<usize> {
        self.fd_mut(fd)?.data.read(buf)
    }

    pub fn fseek(&mut self, fd: FileDescriptor, seek: SeekFrom) -> FsResult<u64> {
        self.fd_mut(fd)?.data.seek(seek)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        filesystems::tmpfs::TmpFs,
        io::{DirectoryProvider, Write},
    };

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

    struct SuperFakeFs {
        super_fake_stuff: usize,
    }

    impl SuperFakeFs {
        fn new() -> Self {
            Self {
                super_fake_stuff: 0,
            }
        }
    }

    impl FileSystemProvider for SuperFakeFs {
        fn open_directory(
            &mut self,
            path: crate::path::Path,
        ) -> FsResult<qk_alloc::boxed::Box<dyn DirectoryProvider>> {
            todo!()
        }
        fn open_file(
            &mut self,
            path: crate::path::Path,
        ) -> FsResult<qk_alloc::boxed::Box<dyn FileProvider>> {
            Err(FsError::new(
                FsErrorKind::StorageFull,
                "Fake storage does not exist",
            ))
        }

        fn mkdir(&mut self, path: crate::path::Path, permission: Permissions) -> FsResult<()> {
            todo!()
        }
        fn rmdir(&mut self, path: crate::path::Path) -> FsResult<()> {
            todo!()
        }

        fn touch(&mut self, path: crate::path::Path, permission: Permissions) -> FsResult<()> {
            todo!()
        }
        fn rm(&mut self, path: crate::path::Path) -> FsResult<()> {
            Err(FsError::new(FsErrorKind::Other, path.as_str().into()))
        }
    }

    #[test]
    fn test_new_with_fake_mount_vfs() {
        crate::set_example_allocator();

        let mut vfs = Vfs::new();
        assert_eq!(
            vfs.mount(Path::from("/"), Box::new(SuperFakeFs::new())),
            Ok(0)
        );
        assert_eq!(
            vfs.mount(Path::from("/test"), Box::new(SuperFakeFs::new())),
            Ok(1)
        );
    }

    #[test]
    fn test_vfs_with_unmount() {
        crate::set_example_allocator();

        let mut vfs = Vfs::new();
        assert_eq!(
            vfs.mount(Path::from("/"), Box::new(SuperFakeFs::new())),
            Ok(0)
        );
        assert_eq!(
            vfs.mount(Path::from("/test"), Box::new(SuperFakeFs::new())),
            Ok(1)
        );

        assert_eq!(vfs.umount("/test".into()).map(|_| ()), Ok(()));
        assert!(vfs.unmount_id(0).is_ok());
    }

    #[test]
    fn test_vfs_fail_mount() {
        crate::set_example_allocator();

        let mut vfs = Vfs::new();
        assert_eq!(vfs.mount("/".into(), Box::new(SuperFakeFs::new())), Ok(0));
        assert!(vfs.mount("/".into(), Box::new(SuperFakeFs::new())).is_err());
    }

    #[test]
    fn test_vfs_read() {
        crate::set_example_allocator();

        let mut vfs = Vfs::new();
        assert_eq!(vfs.mount("/".into(), Box::new(SuperFakeFs::new())), Ok(0));

        assert_eq!(
            vfs.open("/somefile.txt".into())
                .expect_err("Error was OK, should be error")
                .kind(),
            FsErrorKind::StorageFull
        );
    }

    #[test]
    fn test_vfs_path_resl_correctly() {
        crate::set_example_allocator();

        let mut vfs = Vfs::new();
        assert_eq!(
            vfs.mount(Path::from("/"), Box::new(SuperFakeFs::new())),
            Ok(0)
        );
        assert_eq!(
            vfs.mount(Path::from("/test"), Box::new(SuperFakeFs::new())),
            Ok(1)
        );

        assert_eq!(
            vfs.rm("/test/fs.txt".into()).unwrap_err().into_inner(),
            "/fs.txt"
        );

        assert_eq!(
            vfs.rm("/fs.txt".into()).unwrap_err().into_inner(),
            "/fs.txt"
        );

        assert_eq!(
            vfs.rm("/still_root/fs.txt".into())
                .unwrap_err()
                .into_inner(),
            "/still_root/fs.txt"
        );
    }

    #[test]
    fn test_vfs_file() {
        crate::set_example_allocator();

        let mut vfs = Vfs::new();

        assert_eq!(
            vfs.mount(Path::from("/"), Box::new(SuperFakeFs::new())),
            Ok(0)
        );
    }

    #[test]
    fn test_vfs_with_tmpfs_file() {
        crate::set_example_allocator();
        use crate::io::{Read, Seek};

        let mut vfs = Vfs::new();

        assert_eq!(
            vfs.mount(Path::from("/"), Box::new(TmpFs::new(Permissions::all()))),
            Ok(0)
        );

        vfs.touch("/myfile.txt".into(), Permissions::all()).unwrap();
        {
            let mut file = vfs.open("/myfile.txt".into()).unwrap().link_vfs(&mut vfs);

            assert_eq!(file.write(b"123456789"), Ok(9));
            let mut read_buf = [0u8; 9];
            file.rewind().unwrap();
            assert_eq!(file.read(&mut read_buf), Ok(9));
            assert_eq!(&read_buf, b"123456789");
            file.close().unwrap();
        }

        assert_eq!(vfs.open_ids.len(), 0);
        assert_eq!(vfs.open_dds.len(), 0);

        vfs.rm("/myfile.txt".into()).unwrap();
        assert!(matches!(vfs.open("/myfile.txt".into()), Err(_)));
    }

    #[test]
    fn test_vfs_with_tmpfs_dir() {
        crate::set_example_allocator();
        use crate::io::Read;

        let mut vfs = Vfs::new();

        assert_eq!(
            vfs.mount("/".into(), Box::new(TmpFs::new(Permissions::all()))),
            Ok(0)
        );

        vfs.mkdir("/test/".into(), Permissions::all()).unwrap();
        vfs.touch("/test/myfile.txt".into(), Permissions::all())
            .unwrap();
        vfs.touch("/myfile.txt".into(), Permissions::all()).unwrap();

        {
            let mut dir = vfs.open_dir("/test/".into()).unwrap().link_vfs(&mut vfs);
            assert_eq!(dir.next(), Some("/test/myfile.txt".into()));
            assert_eq!(dir.next(), None);
            dir.close().unwrap();
        }

        {
            let mut inner = vfs
                .open("/test/myfile.txt".into())
                .unwrap()
                .link_vfs(&mut vfs);

            assert_eq!(inner.write(b"Hello"), Ok(5));
            inner.flush().unwrap();
            inner.close().unwrap();
        }

        {
            let mut outer = vfs.open("/myfile.txt".into()).unwrap().link_vfs(&mut vfs);

            assert_eq!(outer.write(b"World"), Ok(5));
            outer.flush().unwrap();
            outer.close().unwrap();
        }

        let mut reading_buf = [0u8; 5];
        {
            let mut inner_check = vfs
                .open("/test/myfile.txt".into())
                .unwrap()
                .link_vfs(&mut vfs);
            assert_eq!(inner_check.read(&mut reading_buf), Ok(5));
            assert_eq!(&reading_buf, b"Hello");
            inner_check.close().unwrap();
        }

        {
            let mut outer_check = vfs.open("/myfile.txt".into()).unwrap().link_vfs(&mut vfs);
            assert_eq!(outer_check.read(&mut reading_buf), Ok(5));
            assert_eq!(&reading_buf, b"World");
            outer_check.close().unwrap();
        }

        vfs.rm("/test/myfile.txt".into()).unwrap();
        vfs.rmdir("/test/".into()).unwrap();
        vfs.rm("/myfile.txt".into()).unwrap();
    }
}
