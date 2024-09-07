#![no_std]

use bios::{
    memory::MemoryEntry,
    video::{VesaMode, VesaModeId},
};

/// # Max Memory Map Entires
/// This is the max number of entries that can fit in the Stage-to-Stage info block.
///
/// ONLY USED FOR `MemoryEntry`!
pub const MAX_MEMORY_MAP_ENTRIES: usize = 16;

/// # Stage16 to Stage32 Info Block
/// Used for sending data between these stages.
#[repr(C)]
pub struct Stage16toStage32 {
    pub stage64_ptr: u64,
    pub kernel_ptr: u64,
    pub memory_map: [MemoryEntry; MAX_MEMORY_MAP_ENTRIES],
    pub video_mode: (VesaModeId, VesaMode),
}
