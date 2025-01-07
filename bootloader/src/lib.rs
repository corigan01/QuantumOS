/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
    Part of the Quantum OS Project

Copyright 2024 Gavin Kellam

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

#![no_std]

use bios::{
    memory::MemoryEntry,
    video::{VesaMode, VesaModeId},
};
use mem::phys::PhysMemoryMap;

/// Amount of regions contained in the inital phys memory map.
pub const MEMORY_REGIONS: usize = 64;

/// Kernel fn ptr
pub type KernelEntryFn = extern "C" fn(u64) -> !;

/// # Max Memory Map Entries
/// This is the max number of entries that can fit in the Stage-to-Stage info block.
///
/// ONLY USED FOR `MemoryEntry`!
pub const MAX_MEMORY_MAP_ENTRIES: usize = 16;

/// # `Stage16` to `Stage32` Info Block
/// Used for sending data between these stages.
#[repr(C)]
pub struct Stage16toStage32 {
    pub bootloader_stack_ptr: (u64, u64),
    pub stage32_ptr: (u64, u64),
    pub stage64_ptr: (u64, u64),
    pub kernel_ptr: (u64, u64),
    pub memory_map: [MemoryEntry; MAX_MEMORY_MAP_ENTRIES],
    pub video_mode: Option<(VesaModeId, VesaMode)>,
}

/// # `Stage32` to `Stage64` Info Block
/// Used for sending data between these stages.
#[repr(C)]
pub struct Stage32toStage64 {
    pub bootloader_stack_ptr: (u64, u64),
    pub stage32_ptr: (u64, u64),
    pub stage64_ptr: (u64, u64),
    pub kernel_ptr: (u64, u64),
    pub memory_map: [MemoryEntry; MAX_MEMORY_MAP_ENTRIES],
    pub video_mode: Option<(VesaModeId, VesaMode)>,
}

/// # `Stage64` to `Kernel` Info Block
#[derive(Debug)]
pub struct KernelBootHeader {
    pub phys_mem_map: &'static PhysMemoryMap<MEMORY_REGIONS>,
    pub video_mode: Option<(VesaModeId, VesaMode)>,
}
