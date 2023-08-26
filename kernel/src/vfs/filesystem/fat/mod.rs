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

mod structures;

use qk_alloc::boxed::Box;
use quantum_lib::debug_println;
use crate::vfs::io::IOResult;
use crate::vfs::{VFSPartition, VFSPartitionID};
use crate::vfs::filesystem::fat::structures::bios_block::BiosParameterBlock;
use crate::vfs::filesystem::fat::structures::{ClusterId, FileEntry};


pub struct FatFilesystem {
    partition_id: VFSPartitionID,
    bpb: BiosParameterBlock,
}

impl FatFilesystem {
    pub fn get_root_entry(&self) -> FileEntry {
        self.bpb.get_root_entry()
    }



    pub fn read_cluster(&self, cluster_id: ClusterId, data: &mut [u8]) -> IOResult<()> {
        todo!("Read the cluster contents")
    }

}


pub fn init_fat_fs(partition_id: VFSPartitionID) -> IOResult<Box<dyn VFSPartition>> {
    let bpb = BiosParameterBlock::populate_from_media(
        &mut partition_id.get_entry_mut().partition
    )?;

    let fat_fs = FatFilesystem {
        partition_id,
        bpb
    };

    let root_entry = fat_fs.get_root_entry();

    debug_println!("{:#?}", root_entry);





    todo!("Finish Fat")
}

pub fn is_media_fat_formatted(media: &mut Box<dyn VFSPartition>) -> IOResult<bool> {
    Ok(BiosParameterBlock::populate_from_media(media).is_ok())
}