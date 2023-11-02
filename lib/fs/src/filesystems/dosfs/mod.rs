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

use crate::abstract_buffer::AbstractBuffer;
use crate::filesystems::dosfs::structures::bpb::{BiosParameterBlock, WhereIsRoot};
use crate::filesystems::dosfs::structures::fat::{FatEntry, FileAllocationTable};
use crate::filesystems::dosfs::structures::file_directory::DirectoryEntry;
use crate::filesystems::dosfs::structures::ClusterID;
use crate::io::{Read, Seek, SeekFrom};
use crate::FsResult;
use core::mem::size_of;
use qk_alloc::boxed::Box;
use qk_alloc::vec;
use qk_alloc::vec::Vec;

mod structures;

pub struct Dosfs {
    fat: FileAllocationTable,
    bpb: Box<BiosParameterBlock>,
    buf: AbstractBuffer,
}

impl Dosfs {
    pub fn new(mut buf: AbstractBuffer) -> FsResult<Self> {
        let mut bpb = [0_u8; 512];
        buf.seek(SeekFrom::Start(0))?;
        buf.read(&mut bpb)?;

        let bpb = Box::new(BiosParameterBlock::try_from(bpb.as_ref())?);

        let fat = FileAllocationTable::new(
            bpb.fat_type(),
            bpb.fat_begin_bytes() as u64,
            bpb.fat_size_bytes() as u64,
        );

        Ok(Self { fat, bpb, buf })
    }

    pub fn read_directory(&mut self, cluster_id: ClusterID) -> FsResult<Vec<DirectoryEntry>> {
        let mut dir_entries = Vec::new();
        let mut next_cluster = FatEntry::NextCluster(cluster_id);
        while !next_cluster.is_last() {
            let FatEntry::NextCluster(last_cluster) = next_cluster else {
                break;
            };

            next_cluster = self.fat.read_entry(last_cluster, &mut self.buf)?;
            let volume_offset = self.bpb.preform_cluster_offset(last_cluster)?;
            let cluster_size = self.bpb.bytes_per_cluster();

            let mut buffer = vec![0_u8; cluster_size];

            self.buf.seek(SeekFrom::Start(volume_offset))?;
            self.buf.read(buffer.as_mut())?;

            let entry = DirectoryEntry::try_from(buffer.as_ref())?;
            dir_entries.push(entry);
        }

        Ok(dir_entries)
    }

    fn read_root16(&mut self, root_offset: usize) -> FsResult<Vec<DirectoryEntry>> {
        let root_entries = self.bpb.root_entries();
        let mut directory_entry_vec = Vec::with_capacity(root_entries);

        self.buf.seek(SeekFrom::Start(root_offset as u64))?;
        for _ in 0..root_entries {
            let mut dir_buf = [0u8; size_of::<DirectoryEntry>()];
            self.buf.read(&mut dir_buf)?;

            let directory_entry = DirectoryEntry::try_from(dir_buf.as_ref())?;

            if directory_entry.is_free() {
                continue;
            }

            directory_entry_vec.push(directory_entry);
        }

        Ok(directory_entry_vec)
    }

    pub fn read_root(&mut self) -> FsResult<Vec<DirectoryEntry>> {
        match self.bpb.where_is_root() {
            WhereIsRoot::OffsetBytes(root_offset) => self.read_root16(root_offset),
            WhereIsRoot::Cluster(cluster_id) => self.read_directory(cluster_id),
        }
    }
}
