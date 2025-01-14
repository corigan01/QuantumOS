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

use core::{arch::asm, cell::SyncUnsafeCell};

use arch::{
    gdt::{CodeSegmentDesc, DataSegmentDesc, GlobalDescriptorTable},
    registers::{Segment, SegmentRegisters},
};
use bootgfx::{Color, Framebuffer};
use bootloader::{Stage16toStage32, Stage32toStage64};
use lldebug::{debug_ready, logln, make_debug};
use serial::{baud::SerialBaud, Serial};

mod paging;
mod panic;

static GDT: SyncUnsafeCell<GlobalDescriptorTable<3>> =
    SyncUnsafeCell::new(GlobalDescriptorTable::new());

static S2S: SyncUnsafeCell<Stage32toStage64> = SyncUnsafeCell::new(unsafe { core::mem::zeroed() });

make_debug! {
    "Serial": Option<Serial> = Serial::probe_first(SerialBaud::Baud115200);
}

#[no_mangle]
#[link_section = ".start"]
extern "C" fn _start(stage_to_stage: u32) {
    main(unsafe { &(*(stage_to_stage as *const Stage16toStage32)) });
    panic!("Main should not return");
}

#[debug_ready]
fn main(stage_to_stage: &Stage16toStage32) {
    if let Some(video_mode) = stage_to_stage.video_mode {
        let mut framebuffer = unsafe {
            Framebuffer::new_linear(
                video_mode.1.framebuffer as *mut u32,
                32,
                video_mode.1.height as usize,
                video_mode.1.width as usize,
            )
        };

        framebuffer.draw_rec(
            1,
            1,
            framebuffer.width(),
            framebuffer.height(),
            Color::QUANTUM_BACKGROUND,
        );

        framebuffer.draw_glyph(10, 10, 'Q', Color::WHITE);
        framebuffer.draw_glyph(20, 10, 'O', Color::WHITE);
        framebuffer.draw_glyph(30, 10, 'S', Color::WHITE);
    }

    unsafe { paging::enable_paging() };

    // load gdt
    unsafe {
        let gdt = &mut *GDT.get();

        gdt.store(
            1,
            CodeSegmentDesc::new64()
                .set_accessed_flag(true)
                .set_present_flag(true)
                .set_writable_flag(true),
        );
        gdt.store(
            2,
            DataSegmentDesc::new64()
                .set_accessed_flag(true)
                .set_present_flag(true)
                .set_writable_flag(true),
        );

        // load
        gdt.pack().load();
        logln!("Loaded long mode GDT!");
    }

    // build s2s
    unsafe {
        let s2s = &mut *S2S.get();

        s2s.bootloader_stack_ptr = stage_to_stage.bootloader_stack_ptr;
        s2s.stage32_ptr = stage_to_stage.stage32_ptr;
        s2s.stage64_ptr = stage_to_stage.stage64_ptr;
        s2s.kernel_ptr = stage_to_stage.kernel_ptr;
        s2s.initfs_ptr = stage_to_stage.initfs_ptr;
        s2s.memory_map = stage_to_stage.memory_map;
        s2s.video_mode = stage_to_stage.video_mode.clone();

        logln!("Built Stage32to64!");
    }

    // jump to stage64
    logln!(
        "Jumping to stage64! -- 0x{:016x}",
        stage_to_stage.stage64_ptr.0
    );
    unsafe { enter_stage3(stage_to_stage.stage64_ptr.0 as *const (), S2S.get()) };
}

#[unsafe(no_mangle)]
pub unsafe fn enter_stage3(entry_ptr: *const (), s2s: *const Stage32toStage64) {
    SegmentRegisters::set_data_segments(Segment::new(2, arch::CpuPrivilege::Ring0));

    asm!("ljmp $0x8, $2f", "2:", options(att_syntax));
    asm!(
        ".code64",
        "call rax",
        in("rax") entry_ptr,
        in("rdi") s2s
    );
}
