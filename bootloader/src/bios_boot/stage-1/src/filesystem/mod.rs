/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
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

pub mod types;

pub mod fat;
pub mod partition;

use core::marker::PhantomData;
use types::FileSystemTypes;
use crate::bios_disk::BiosDisk;
use crate::error::BootloaderError;
use crate::filesystem::partition::Partitions;
use crate::filesystem::types::FileSystemTypes::Fat;

pub struct UnQuarried;
pub struct Quarried;

pub trait DiskMedia {
    // FIXME: sector size should not be defined to always 512
    fn read(&self, sector: usize) -> Result<[u8; 512], BootloaderError>;
}

pub struct FileSystem<DiskType, State = UnQuarried>
    where DiskType: Sized + DiskMedia {
    // FIXME: Should not be a hard defined limit on how many filesystems can be attached
    current_filesystem: [FileSystemTypes; 4],
    attached_disk: DiskType,

    state: PhantomData<State>
}

impl<DiskType: DiskMedia> FileSystem<DiskType> {
    pub fn new(disk: DiskType) -> FileSystem<DiskType> {
        FileSystem {
            current_filesystem: [FileSystemTypes::new(); 4],
            attached_disk: disk,

            state: Default::default(),
        }
    }

    fn read_from_disk(&self, sector: usize) -> Result<[u8; 512], BootloaderError> {
        self.attached_disk.read(sector)
    }
}

impl <DiskType: DiskMedia> FileSystem<DiskType, UnQuarried> {
    pub fn quarry_disk(self) -> Result<FileSystem<DiskType, Quarried>, BootloaderError> {
        let partitions = Partitions::check_all(self.attached_disk)?;
        let entries_ref = partitions.get_partitions_ref();

        for partition in entries_ref {

        }

        Err(BootloaderError::NoValid)
    }
}


impl <DiskType: DiskMedia> FileSystem<DiskType, Quarried> {

}


fn test() {
    let disk = BiosDisk::new(0x80);

    let fs = FileSystem::<BiosDisk>::new(disk);
    let fs = fs.quarry_disk().unwrap();









}

/*
fn test() {
    let disk = BiosDisk::new(0x80);
    let fs = FileSystem::new();

    fs.quarry_disk(disk);

    if let Some(partition) = fs.quarry_for_item("bootloader.conf").bootable() {
        fs.attach_root(partition).unwrap();

        assert!(fs.does_item_exist("*kernel*"));
        assert!(fs.does_item_exist("bootloader.conf"));

    } else {
        panic!("Can not find required bootable fields.");
    }

    let bootloader_config = fs.open_file("bootloader.conf");
    let bootloader_config = bootloader_config.read_all().unwrap();

    struct BiosConfigurator;
    impl BiosConfigurator {
        pub fn new(x: _) -> Self {
            Self {}
        }
    }

    let config = BiosConfigurator::new(bootloader_config.to_string());
    let kernel_start = config.quarry_key_int("kernel.start").unwarp_or(0);

}*/