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

use crate::e820_memory::E820Entry;
use crate::BootMemoryDescriptor;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct SimpleRamFs {
    pub kernel: BootMemoryDescriptor,
    pub stage2: BootMemoryDescriptor,
    pub stage3: BootMemoryDescriptor,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct VideoInformation {
    pub video_mode: u16,
    pub x: u32,
    pub y: u32,
    pub depth: u32,
    pub framebuffer: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(packed, C)]
pub struct BootInfo {
    pub booted_disk_id: u16,
    pub ram_fs: SimpleRamFs,
    pub vid: VideoInformation,
    pub memory_map_ptr: u64,
    pub memory_map_size: u64,
    pub magic: u64,
}

impl BootInfo {
    pub const MAGIC: u64 = 0xdeadbeef;

    pub fn new() -> Self {
        Self {
            booted_disk_id: 0,
            ram_fs: SimpleRamFs::default(),
            vid: VideoInformation::default(),
            memory_map_ptr: 0,
            memory_map_size: 0,
            magic: Self::MAGIC,
        }
    }

    pub fn from_ptr<'a>(ptr: usize) -> &'a Self {
        let self_ref = unsafe { &*(ptr as *const Self) };
        let magic = self_ref.magic;

        assert!(
            ptr >= 0x100000,
            "BootInfo Struct Malformed - The address is not what is expected 0x{:x}",
            ptr
        );
        assert!(
            self_ref.check_magic(),
            "BootInfo Struct Magic FAILED, Got=0x{:x} Expected=0x{:x}",
            magic,
            Self::MAGIC
        );

        self_ref
    }

    pub fn check_magic(&self) -> bool {
        Self::MAGIC == self.magic
    }

    pub fn set_booted_disk_id(&mut self, id: u16) {
        self.booted_disk_id = id;
    }

    pub fn set_kernel_entry(&mut self, disc: BootMemoryDescriptor) {
        self.ram_fs.kernel = disc;
    }

    pub fn set_stage_2_entry(&mut self, disc: BootMemoryDescriptor) {
        self.ram_fs.stage2 = disc;
    }

    pub fn set_stage_3_entry(&mut self, disc: BootMemoryDescriptor) {
        self.ram_fs.stage3 = disc;
    }

    pub fn set_video_information(&mut self, video: VideoInformation) {
        self.vid = video;
    }

    pub fn set_memory_map(&mut self, ptr: *const E820Entry, size: usize) {
        self.memory_map_ptr = ptr as u64;
        self.memory_map_size = size as u64;
    }

    pub fn get_booted_disk_id(&self) -> usize {
        self.booted_disk_id as usize
    }

    pub fn get_kernel_entry(&self) -> BootMemoryDescriptor {
        self.ram_fs.kernel
    }

    pub fn get_stage_2_entry(&self) -> BootMemoryDescriptor {
        self.ram_fs.stage2
    }

    pub fn get_stage_3_entry(&self) -> BootMemoryDescriptor {
        self.ram_fs.stage3
    }

    pub fn get_video_information(&self) -> VideoInformation {
        self.vid
    }

    pub unsafe fn get_memory_map(&self) -> &[E820Entry] {
        core::slice::from_raw_parts(
            self.memory_map_ptr as *const E820Entry,
            self.memory_map_size as usize,
        )
    }
}

impl Default for BootInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleRamFs {
    pub fn new(
        kernel: BootMemoryDescriptor,
        stage2: BootMemoryDescriptor,
        stage3: BootMemoryDescriptor,
    ) -> Self {
        Self {
            kernel,
            stage2,
            stage3,
        }
    }
}
