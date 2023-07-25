/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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

use core::fmt::{Debug, Display, Formatter};
use core::slice::{Iter, IterMut};
use qk_alloc::vec::Vec;
use qk_alloc::boxed::Box;
use quantum_lib::debug_println;
use crate::vfs::ata::init_ata_disks;
use crate::vfs::io::{DiskInfo, Read, Seek, Write};

pub mod ata;
pub mod filesystem;
pub mod partitioning;
pub mod io;

pub trait VFSDisk: DiskInfo + Seek + Read + Write {}
impl<T> VFSDisk for T
    where T: DiskInfo + Seek + Read + Write {}

impl Display for dyn VFSDisk {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Disk Info")
            .field("Disk Name", &self.disk_model().as_str())
            .finish()?;

        Ok(())
    }
}

static mut VFS_DISK_NEXT_ID: usize = 0;
static mut VFS_DISKS: Vec<(VFSDiskID, Box<dyn VFSDisk>)> = Vec::new();

pub struct VFSDiskContainer {
    raw_disk: Box<dyn VFSDisk>,
    partitions: Vec<>
}

#[derive(Clone, Copy)]
pub(crate) struct VFSDiskID(usize);

impl VFSDiskID {
    pub fn publish_disk(disk: Box<dyn VFSDisk>) -> VFSDiskID {
        let disk_id;
        unsafe {
            disk_id = Self(VFS_DISK_NEXT_ID);

            debug_println!("VFS Registered new disk [id: {}] ('{}', {})", VFS_DISK_NEXT_ID, disk.disk_model().as_str(), disk.disk_capacity());

            VFS_DISKS.push((
                disk_id,
                disk
                ));

            VFS_DISK_NEXT_ID += 1;
        }

        disk_id
    }

    pub fn disks_iter() -> Iter<'static, (VFSDiskID, Box<dyn VFSDisk>)> {
        unsafe { VFS_DISKS.iter() }
    }

    pub fn disks_iter_mut() -> IterMut<'static, (VFSDiskID, Box<dyn VFSDisk>)> {
        unsafe { VFS_DISKS.iter_mut() }
    }

    pub fn get_ref(&self) -> Option<&Box<dyn VFSDisk>> {
        Self::disks_iter().find(|(id, _disk)| {
            id.0 == self.0
        }).map(|(_id, disk)| disk)
    }

    pub fn get_mut(&self) -> Option<&mut Box<dyn VFSDisk>> {
        Self::disks_iter_mut().find(|(id, _disk)| {
            id.0 == self.0
        }).map(|(_id, disk)| disk)
    }
}

pub fn init() {
    debug_println!("\n\nVFS -----------------");
    init_ata_disks();


    debug_println!("\n\n---------------------");
}