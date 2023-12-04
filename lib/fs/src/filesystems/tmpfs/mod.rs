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

use self::{
    dir::TmpDirectory,
    file::{TmpFile, TmpOpenFile},
};
use crate::{
    error::FsError, filesystems::tmpfs::dir::TmpOpenDirectory, io::FileSystemProvider, path::Path,
    permission::Permissions, FsResult,
};
use qk_alloc::{boxed::Box, vec::Vec};

mod dir;
mod file;

pub struct TmpFs {
    files: Vec<Box<TmpFile>>,
    dires: Vec<Box<TmpDirectory>>,
}

impl TmpFs {
    pub fn new(root_perm: Permissions) -> Self {
        let mut dirs = Vec::new();
        let root_dir = TmpDirectory::new("/".into(), root_perm);
        dirs.push(Box::new(root_dir));

        Self {
            files: Vec::new(),
            dires: dirs,
        }
    }

    fn get_dir_index_for_path(&mut self, path: &Path) -> FsResult<usize> {
        self.dires
            .iter()
            .enumerate()
            .find(|(_, entry)| {
                entry.path.as_str().trim_end_matches("/") == path.as_str().trim_end_matches("/")
            })
            .map(|(index, _)| index)
            .ok_or(FsError::new(
                crate::error::FsErrorKind::NotFound,
                "That directory was not found!",
            ))
    }

    fn get_file_index_for_path(&mut self, path: &Path) -> FsResult<usize> {
        self.files
            .iter()
            .enumerate()
            .find(|(_, entry)| &entry.path == path)
            .map(|(index, _)| index)
            .ok_or(FsError::new(
                crate::error::FsErrorKind::NotFound,
                "That file was not found!",
            ))
    }

    fn does_parent_exist(&self, path: &Path) -> bool {
        self.dires.iter().any(|entry| path.is_child_of(&entry.path)) || path.parent_path().is_root()
    }

    fn get_directory_parent(&mut self, path: &Path) -> &mut TmpDirectory {
        let path = path.parent_path();
        self.dires
            .iter_mut()
            .find(|entry| {
                path.as_str().trim_end_matches("/") == entry.path.as_str().trim_end_matches("/")
            })
            .expect("It should not be possible to fail when we just checked if we have a parent!")
    }

    fn remove_directory_entry(&mut self, path: &Path) {
        self.get_directory_parent(&path)
            .entries
            .retain(|entry| entry.as_str() != path.as_str());
    }

    fn add_directory_entry(&mut self, path: &Path) {
        self.get_directory_parent(&path)
            .entries
            .push(path.truncate_path());
    }
}

impl FileSystemProvider for TmpFs {
    fn open_directory(
        &mut self,
        path: crate::path::Path,
    ) -> crate::FsResult<qk_alloc::boxed::Box<dyn crate::io::DirectoryProvider>> {
        let dir_index = self.get_dir_index_for_path(&path)?;
        let tmp_open = TmpOpenDirectory::from(self.dires[dir_index].as_ref());

        Ok(Box::new(tmp_open))
    }

    fn open_file(
        &mut self,
        path: crate::path::Path,
    ) -> crate::FsResult<qk_alloc::boxed::Box<dyn crate::io::FileProvider>> {
        let file_index = self.get_file_index_for_path(&path)?;
        let tmp_file = TmpOpenFile::from(&mut self.files[file_index]);

        Ok(Box::new(tmp_file))
    }

    fn rmdir(&mut self, path: crate::path::Path) -> crate::FsResult<()> {
        let dir_index = self.get_dir_index_for_path(&path)?;

        // The scope of this refrence will no longer be valid when we move the entry
        // thats why we enclose it in this block so the refrence is dropped before our
        // value is deleted.
        {
            let dir_ref = &self.dires[dir_index];
            if dir_ref.entries.len() > 0 {
                return Err(FsError::new(
                crate::error::FsErrorKind::PermissionDenied,
                "Cannot remove a directory with children, remove children before deleting the directory!",
            ));
            }
        }

        self.remove_directory_entry(&path);
        self.dires.remove(dir_index);
        Ok(())
    }

    fn rm(&mut self, path: crate::path::Path) -> crate::FsResult<()> {
        let file_index = self.get_file_index_for_path(&path)?;

        self.remove_directory_entry(&path);
        self.files.remove(file_index);
        Ok(())
    }

    fn mkdir(
        &mut self,
        path: crate::path::Path,
        permission: crate::permission::Permissions,
    ) -> crate::FsResult<()> {
        if self.get_dir_index_for_path(&path).is_ok() {
            return Err(FsError::new(
                crate::error::FsErrorKind::AlreadyExists,
                "The directory already exists at that path!",
            ));
        }

        if !self.does_parent_exist(&path) {
            return Err(FsError::new(
                crate::error::FsErrorKind::InvalidInput,
                "The parent for this directory does not exist!",
            ));
        }

        self.add_directory_entry(&path);

        let new_dir = TmpDirectory::new(path, permission);
        self.dires.push(Box::new(new_dir));

        Ok(())
    }

    fn touch(
        &mut self,
        path: crate::path::Path,
        permission: crate::permission::Permissions,
    ) -> crate::FsResult<()> {
        if self.get_file_index_for_path(&path).is_ok() {
            return Err(FsError::new(
                crate::error::FsErrorKind::AlreadyExists,
                "The file already exists at that path!",
            ));
        }

        if !self.does_parent_exist(&path) {
            return Err(FsError::new(
                crate::error::FsErrorKind::InvalidInput,
                "The parent for this file does not exist!",
            ));
        }

        self.add_directory_entry(&path);

        let new_file = TmpFile::new(path.clone(), permission);
        self.files.push(Box::new(new_file));
        Ok(())
    }

    fn supports_permissions(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod test {
    use crate::permission::Permissions;

    use super::*;

    fn setup_test() -> crate::Vfs {
        crate::set_example_allocator();

        let tmpfs = TmpFs::new(Permissions::all());
        let mut vfs = crate::Vfs::new();
        assert_eq!(vfs.mount("/".into(), Box::new(tmpfs)), Ok(0));

        vfs
    }

    #[test]
    fn test_tmp_dir_new() {
        let mut vfs = setup_test();
        assert!(matches!(vfs.unmount_id(0), Ok(_)));
    }

    #[test]
    fn test_tmp_dir_newfile() {
        crate::set_example_allocator();
        let mut tmpfs = TmpFs::new(Permissions::all());

        tmpfs.touch("/test.txt".into(), Permissions::all()).unwrap();
    }

    #[test]
    fn test_tmp_dir_newdir() {
        crate::set_example_allocator();
        let mut tmpfs = TmpFs::new(Permissions::all());

        tmpfs.mkdir("/test/".into(), Permissions::all()).unwrap();
    }

    #[test]
    fn test_tmp_dir_create_file_read_and_write() {
        crate::set_example_allocator();
        let mut tmpfs = TmpFs::new(Permissions::all());

        tmpfs.touch("/test.txt".into(), Permissions::all()).unwrap();
        let mut file = tmpfs.open_file("/test.txt".into()).unwrap();

        file.write(b"Hello World!").unwrap();
        file.flush().unwrap();

        file.seek(crate::io::SeekFrom::Start(0)).unwrap();
        let mut read_buff = [0_u8; 12];
        assert_eq!(file.read(&mut read_buff), Ok(12));

        assert_eq!(&read_buff, b"Hello World!");
    }

    #[test]
    fn test_tmp_dir_create_dir_and_file() {
        crate::set_example_allocator();
        let mut tmpfs = TmpFs::new(Permissions::all());

        tmpfs.mkdir("/dir".into(), Permissions::all()).unwrap();
        tmpfs
            .touch("/dir/test.txt".into(), Permissions::all())
            .unwrap();
    }

    #[test]
    fn test_tmp_dir_create_dir_and_del() {
        crate::set_example_allocator();
        let mut tmpfs = TmpFs::new(Permissions::all());

        tmpfs.mkdir("/mydir".into(), Permissions::all()).unwrap();
        tmpfs
            .touch("/mydir/test.txt".into(), Permissions::all())
            .unwrap();

        assert!(
            tmpfs.rmdir("/mydir".into()).is_err(),
            "It should not be possible to remove dirs with children"
        );
        tmpfs.rm("/mydir/test.txt".into()).unwrap();
        assert!(
            tmpfs.rmdir("/mydir".into()).is_ok(),
            "It should be possible to remove a dir with no children"
        );
    }

    #[test]
    fn test_tmp_dir_create_dir_dir_del() {
        crate::set_example_allocator();
        let mut tmpfs = TmpFs::new(Permissions::all());

        tmpfs.mkdir("/test/".into(), Permissions::all()).unwrap();
        tmpfs
            .mkdir("/test/test/".into(), Permissions::all())
            .unwrap();
        tmpfs
            .mkdir("/test/test/testfinal".into(), Permissions::all())
            .unwrap();

        assert!(
            tmpfs.rmdir("/test".into()).is_err(),
            "It should not be possible to remove a dir with children"
        );

        tmpfs.rmdir("/test/test/testfinal".into()).unwrap();
        tmpfs.rmdir("/test/test/".into()).unwrap();
        tmpfs.rmdir("/test".into()).unwrap();
    }

    #[test]
    fn test_tmp_dir_crate_dir_files() {
        crate::set_example_allocator();
        let mut tmpfs = TmpFs::new(Permissions::all());

        tmpfs.mkdir("/myfiles".into(), Permissions::all()).unwrap();
        tmpfs
            .mkdir("/myfiles/1".into(), Permissions::all())
            .unwrap();
        tmpfs
            .touch("/myfiles/test.txt".into(), Permissions::all())
            .unwrap();
        {
            let mut file = tmpfs.open_file("/myfiles/test.txt".into()).unwrap();
            file.write(b"Hello World").unwrap();
            assert_eq!(file.stream_len(), Ok(11));
        }

        tmpfs.rmdir("/myfiles/1".into()).unwrap();

        {
            let mut file = tmpfs.open_file("/myfiles/test.txt".into()).unwrap();
            assert_eq!(file.stream_len(), Ok(11));
            let mut string = [0u8; 11];
            assert_eq!(file.read(&mut string), Ok(11));
            assert_eq!(&string, b"Hello World");
        }

        tmpfs.rm("/myfiles/test.txt".into()).unwrap();
        tmpfs.rmdir("/myfiles".into()).unwrap();
    }
}
