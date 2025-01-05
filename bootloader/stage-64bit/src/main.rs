/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

#![no_main]
#![no_std]
#![feature(sync_unsafe_cell)]

use bootloader::Stage32toStage64;
use core::cell::SyncUnsafeCell;
use elf::{
    Elf,
    tables::{ArchKind, SegmentKind},
};
use lldebug::{debug_ready, logln, make_debug};
use mem::phys::{PhysMemoryEntry, PhysMemoryKind, PhysMemoryMap};
use serial::{Serial, baud::SerialBaud};
use util::{
    align_to,
    bytes::HumanBytes,
    consts::{MIB, PAGE_2M, PAGE_4K},
};

mod paging;
mod panic;

make_debug! {
    "Serial": Option<Serial> = Serial::probe_first(SerialBaud::Baud115200);
}

static MEMORY_MAP: SyncUnsafeCell<PhysMemoryMap<64>> = SyncUnsafeCell::new(PhysMemoryMap::new());

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
extern "C" fn _start(stage_to_stage: u64) {
    main(unsafe { &(*(stage_to_stage as *const Stage32toStage64)) });
    panic!("Main should not return");
}

#[debug_ready]
fn main(stage_to_stage: &Stage32toStage64) {
    logln!("Stage64!");
    let (kernel_elf_ptr, kernel_elf_size) = stage_to_stage.kernel_ptr;

    let elf = Elf::new(unsafe {
        core::slice::from_raw_parts(kernel_elf_ptr as *const u8, kernel_elf_size as usize)
    });

    let kernel_exe_len = elf
        .exe_size()
        .expect("Unable to determine the size of the Kernel's exe!");

    logln!("Kernel Size: {}", HumanBytes::from(kernel_exe_len));
    build_memory_map(stage_to_stage, kernel_exe_len);

    let elf_header = match elf.header() {
        Ok(elf::tables::ElfHeader::Header64(h)) if h.arch() == ArchKind::X64 && h.is_le() => h,
        _ => panic!("Kernel's elf is not valid!"),
    };

    elf.load_into(|h| {
        if h.segment_kind() != SegmentKind::Load {
            return None;
        }

        None
    })
    .unwrap();
}

fn build_memory_map(s2s: &Stage32toStage64, kernel_exe_len: usize) {
    unsafe {
        let mm = &mut *MEMORY_MAP.get();

        for memory_region in s2s.memory_map.iter() {
            mm.add_region(memory_region)
                .expect("Unable to build kernel's memory map!");
        }

        logln!(
            "Free Memory : {} Mib",
            mm.bytes_of(PhysMemoryKind::Free) / MIB
        );
        logln!(
            "Reserved Memory : {} Mib",
            mm.bytes_of(PhysMemoryKind::Reserved) / MIB
        );

        let (s32_start, s32_len) = s2s.stage32_ptr;
        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Bootloader,
            start: s32_start,
            end: s32_start + s32_len,
        })
        .expect("Unable to add stage32 to memory map");

        let (s64_start, s64_len) = s2s.stage64_ptr;
        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Bootloader,
            start: s64_start,
            end: s64_start + s64_len,
        })
        .expect("Unable to add stage64 to memory map");

        let (stack_start, stack_len) = s2s.bootloader_stack_ptr;
        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Bootloader,
            start: stack_start,
            end: stack_start + stack_len,
        })
        .expect("Unable to add bootloader's stack to memory map");

        let kernels_pages = mm
            .find_continuous_of(
                PhysMemoryKind::Free,
                align_to(kernel_exe_len as u64, PAGE_4K) as usize,
                PAGE_4K,
                1 * MIB as u64,
            )
            .map(|p| PhysMemoryEntry {
                kind: PhysMemoryKind::Kernel,
                ..p
            })
            .expect("Unable to find region for kernel pages");
        mm.add_region(kernels_pages).unwrap();

        let kernels_stack_pages = mm
            .find_continuous_of(PhysMemoryKind::Free, PAGE_2M, PAGE_4K, 1 * MIB as u64)
            .map(|p| PhysMemoryEntry {
                kind: PhysMemoryKind::Kernel,
                ..p
            })
            .expect("Unable to find region for kernel's stack pages");
        mm.add_region(kernels_stack_pages).unwrap();

        logln!("{}", mm);
    }
}
