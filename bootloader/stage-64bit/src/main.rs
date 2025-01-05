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
use mem::phys::{PhysMemoryEntry, PhysMemoryMap};
use serial::{Serial, baud::SerialBaud};

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

    unsafe {
        let mm = &mut *MEMORY_MAP.get();

        for memory_region in stage_to_stage.memory_map.iter() {
            mm.add_region(memory_region)
                .expect("Unable to build kernel's memory map!");
        }

        logln!(
            "Free Memory : {} Mib",
            mm.bytes_of(mem::phys::PhysMemoryKind::Free) / util::consts::MIB
        );
        logln!(
            "Reserved Memory : {} Mib",
            mm.bytes_of(mem::phys::PhysMemoryKind::Reserved) / util::consts::MIB
        );

        let (s32_start, s32_len) = stage_to_stage.stage32_ptr;
        mm.add_region(PhysMemoryEntry {
            kind: mem::phys::PhysMemoryKind::Bootloader,
            start: s32_start,
            end: s32_start + s32_len,
        })
        .expect("Unable to add stage32 to memory map");

        let (s64_start, s64_len) = stage_to_stage.stage64_ptr;
        mm.add_region(PhysMemoryEntry {
            kind: mem::phys::PhysMemoryKind::Bootloader,
            start: s64_start,
            end: s64_start + s64_len,
        })
        .expect("Unable to add stage64 to memory map");

        logln!("{}", mm);
    }

    let elf = Elf::new(unsafe {
        core::slice::from_raw_parts(kernel_elf_ptr as *const u8, kernel_elf_size as usize)
    });

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
