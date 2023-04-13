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

use crate::BootMemoryDescriptor;

#[derive(Clone, Copy, Debug)]
pub struct SimpleRamFs {
    pub kernel: BootMemoryDescriptor,
    pub stage2: BootMemoryDescriptor,
}

#[derive(Clone, Copy, Debug)]
pub struct VideoInformation {
    pub video_mode: u16,
    pub x: u32,
    pub y: u32,
    pub depth: u32,
    pub framebuffer: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct BootInfo {
    pub booted_disk_id: u16,
    pub ram_fs: Option<SimpleRamFs>,
    pub vid: Option<VideoInformation>,
}

impl BootInfo {
    pub fn new() -> Self {
        Self {
            booted_disk_id: 0,
            ram_fs: None,
            vid: None,
        }
    }
}

impl Default for BootInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleRamFs {
    pub fn new(kernel: BootMemoryDescriptor, stage2: BootMemoryDescriptor) -> Self {
        Self { kernel, stage2 }
    }
}
