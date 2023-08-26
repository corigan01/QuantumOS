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

use qk_alloc::string::String;
use crate::vfs::io::IOError;

pub mod bios_block;
pub mod fat_table;

mod raw;

pub trait FatProvider {
    fn fat_type(&self) -> FatType;
    fn volume_label(&self) -> String;
}

#[derive(Clone, Copy, Debug)]
pub enum EntryType {
    RootDir,
    Dir,
    File
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    path: String,
    start_cluster: u64,
    sector_count: u64,
    kind: EntryType
}

pub enum FatType {
    Unknown,
    Fat16,
}

pub enum FatEntry {
    NextCluster(usize),
    BadCluster,
    Unused,
    EndOfFile
}

impl FatEntry {
    pub fn from_fat16_raw(value: usize) -> FatEntry {
        todo!()
    }

    pub fn into_fat16_raw(self) -> usize {
        todo!()
    }
}

pub struct ClusterId(usize);

impl TryFrom<usize> for ClusterId {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < 2 {
            Err(())
        } else {
            Ok(Self(value))
        }
    }
}
impl Into<usize> for ClusterId {
    fn into(self) -> usize {
        self.0
    }
}