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

use core::cell::SyncUnsafeCell;

use arch::gdt::{CodeSegmentDesc, DataSegmentDesc, GlobalDescriptorTable};
use bootgfx::{Color, Framebuffer};
use bootloader::Stage16toStage32;
use lldebug::{debug_ready, make_debug, println};
use serial::{baud::SerialBaud, Serial};

mod paging;
mod panic;

static GDT: SyncUnsafeCell<GlobalDescriptorTable<3>> =
    SyncUnsafeCell::new(GlobalDescriptorTable::new());

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
    let mut framebuffer = unsafe {
        Framebuffer::new_linear(
            stage_to_stage.video_mode.1.framebuffer as *mut u32,
            32,
            stage_to_stage.video_mode.1.height as usize,
            stage_to_stage.video_mode.1.width as usize,
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
        println!("Loaded long mode GDT!");
    }
}
