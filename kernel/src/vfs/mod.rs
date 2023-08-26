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

use crate::vfs::ata::init_ata_disks;
use crate::vfs::io::{DiskInfo, PartitionInfo, Read, Seek, Write};
use crate::vfs::partitioning::init_partitioning_for_disks;
use core::fmt::{Display, Formatter};
use core::slice::{Iter, IterMut};
use qk_alloc::boxed::Box;
use qk_alloc::string::String;
use qk_alloc::vec::Vec;
use quantum_lib::debug_println;
use quantum_utils::human_bytes::HumanBytes;
use crate::vfs::filesystem::{SupportedFilesystem, try_init_on_all};

pub mod ata;
pub mod filesystem;
pub mod io;
pub mod partitioning;

pub trait VFSDisk: DiskInfo + Seek + Read + Write {}
impl<T> VFSDisk for T where T: DiskInfo + Seek + Read + Write {}

impl Display for dyn VFSDisk {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Disk Info")
            .field("Disk Name", &self.disk_model().as_str())
            .finish()?;

        Ok(())
    }
}

pub trait VFSPartition: PartitionInfo + Seek + Read + Write {}
impl<T> VFSPartition for T where T: PartitionInfo + Seek + Read + Write {}

pub trait VFSFilesystem {

}

pub struct VFSFilesystemEntry {
    entry_point: Option<String>,
    filesystem: SupportedFilesystem,
    driver: Box<dyn VFSFilesystem>
}

pub struct VFSPartitionEntry {
    id: VFSPartitionID,
    partition: Box<dyn VFSPartition>,
    filesystem: Option<VFSFilesystemEntry>
}

pub struct VFSEntry {
    id: VFSDiskID,
    disk: Box<dyn VFSDisk>,
    parts: Vec<VFSPartitionEntry>,
}

static mut VFS_DISK_NEXT_ID: usize = 0;
static mut VFS_PARTITION_NEXT_ID: usize = 0;

static mut VFS_ENTRIES: Vec<VFSEntry> = Vec::new();

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct VFSPartitionID(usize);

impl VFSPartitionID {
    pub fn get_entry(&self) -> &VFSPartitionEntry {
        VFSDiskID::resolve_partition_id_reference(self).unwrap()
    }

    pub fn get_entry_mut(&self) -> &mut VFSPartitionEntry {
        VFSDiskID::resolve_partition_id_reference(self).unwrap()
    }

    pub fn resolve_parent_disk(&self) -> VFSDiskID {
        for disks in VFSDiskID::disks_iter() {
            for partitions in disks.parts.iter() {
                if partitions.id == *self {
                    return disks.id;
                }
            }
        }

        unreachable!("Should be able to resolve parent disk for partition")
    }

    pub fn attach_filesystem(&self, filesystem: VFSFilesystemEntry) {
        self.get_entry_mut().filesystem = Some(filesystem);
    }

    pub fn fs_type(&self) -> Option<SupportedFilesystem> {
        Some(self.get_entry().filesystem.as_ref()?.filesystem)
    }

    pub fn mount_point(&self) -> Option<String> {
        self.get_entry().filesystem.as_ref()?.entry_point.clone()
    }
}


#[derive(Clone, Copy, PartialEq)]
pub struct VFSDiskID(usize);

impl VFSDiskID {
    pub fn publish_disk(disk: Box<dyn VFSDisk>) -> VFSDiskID {
        let disk_id;
        unsafe {
            disk_id = Self(VFS_DISK_NEXT_ID);

            debug_println!(
                "VFS Registered new disk [id: {}] ('{}', {}) {}",
                VFS_DISK_NEXT_ID,
                disk.disk_model().as_str(),
                disk.disk_capacity(),
                disk.disk_bus()
            );

            VFS_ENTRIES.push(VFSEntry {
                id: disk_id,
                disk,
                parts: Vec::new(),
            });

            VFS_DISK_NEXT_ID += 1;
        }

        disk_id
    }

    pub fn run_on_all_disk_ids<F>(mut runner: F)
    where
        F: FnMut(&VFSDiskID),
    {
        for entry in Self::disks_iter() {
            runner(&entry.id);
        }
    }

    pub fn publish_partitions(&self, partitions: Vec<Box<dyn VFSPartition>>) {
        let own_entry = self.get_entry_mut();
        debug_println!("VFS Registered {} new partitions on disk {}:", partitions.len(), self.0);
        for partition in partitions.into_iter() {
            debug_println!("    0x{:012x} --> 0x{:012x} ({:10}) -- {}",
                partition.logical_partition_start_byte(),
                partition.logical_partition_end_byte(),
                HumanBytes::from(partition.logical_partition_end_byte() - partition.logical_partition_start_byte()),
                if partition.is_bootable() { "bootable" } else { "" }
            );

            own_entry.parts.push(VFSPartitionEntry {
                id: VFSPartitionID(unsafe { VFS_PARTITION_NEXT_ID }),
                partition,
                filesystem: None,
            });

            unsafe { VFS_PARTITION_NEXT_ID += 1; }
        }
    }

    pub fn resolve_partition_id_reference(id: &VFSPartitionID) -> Option<&mut VFSPartitionEntry> {
        for disks in Self::disks_iter_mut() {
            for partitions in disks.parts.iter_mut() {
                if *id == partitions.id {
                    return Some(partitions);
                }
            }
        }

        return None;
    }

    pub fn run_on_partition_ids<F>(&self, mut runner: F)
        where F: FnMut(&VFSPartitionID) {
        for partition in self.get_entry_mut().parts.iter() {
            runner(&partition.id);
        }
    }

    pub fn disks_iter() -> Iter<'static, VFSEntry> {
        unsafe { VFS_ENTRIES.iter() }
    }

    pub fn disks_iter_mut() -> IterMut<'static, VFSEntry> {
        unsafe { VFS_ENTRIES.iter_mut() }
    }

    pub fn get_entry(&self) -> &VFSEntry {
        Self::disks_iter_mut()
            .find(|disk_entry| disk_entry.id.0 == self.0)
            .expect("Could not find own entry in disk list!")
    }

    pub fn get_entry_mut(&self) -> &mut VFSEntry {
        Self::disks_iter_mut()
            .find(|disk_entry| disk_entry.id.0 == self.0)
            .expect("Could not find own entry in disk list!")
    }

    pub fn get_disk_mut(&self) -> &mut Box<dyn VFSDisk> {
        &mut self.get_entry_mut().disk
    }

    pub fn get_disk(&self) -> &Box<dyn VFSDisk> {
        &self.get_entry_mut().disk
    }
}

pub fn init() {
    debug_println!("\n\nVFS -----------------");
    init_ata_disks();
    init_partitioning_for_disks();
    try_init_on_all();

    debug_println!("\n\n---------------------");
}
