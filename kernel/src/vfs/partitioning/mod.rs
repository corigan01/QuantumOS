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

use crate::vfs::io::{ErrorKind, IOError};
use crate::vfs::partitioning::mbr::init_mbr_for_disk;
use crate::vfs::VFSDiskID;
use core::error::Error;
use core::fmt::{Debug, Display, Formatter};
use qk_alloc::boxed::Box;

pub mod mbr;

#[derive(Debug)]
pub enum PartitionErr {
    Unknown,
    NotSeekable,
    NotValidPartition
}

impl From<PartitionErr> for Box<dyn IOError> {
    fn from(value: PartitionErr) -> Self {
        Box::new(value)
    }
}

impl Error for PartitionErr {}

impl Display for PartitionErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl IOError for PartitionErr {
    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::NotSeekable => ErrorKind::NotSeekable,
            _ => ErrorKind::Unknown,
        }
    }
}

pub fn init_partitioning_for_one_disk(disk: VFSDiskID) {
    init_mbr_for_disk(disk).expect("Unable to partition disk!");
}

pub fn init_partitioning_for_disks() {
    VFSDiskID::run_on_all_disk_ids(|id| {
        init_partitioning_for_one_disk(*id);
    })
}
