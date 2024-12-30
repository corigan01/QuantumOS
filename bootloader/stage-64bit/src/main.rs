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

use bootloader::Stage32toStage64;
use elf::{
    Elf,
    tables::{ArchKind, SegmentKind},
};
use lldebug::{debug_ready, logln, make_debug};
use serial::{Serial, baud::SerialBaud};

mod panic;

make_debug! {
    "Serial": Option<Serial> = Serial::probe_first(SerialBaud::Baud115200);
}

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
    logln!("Memory Map {:#?}", stage_to_stage.memory_map);

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
