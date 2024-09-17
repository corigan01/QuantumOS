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

use bootgfx::{Color, Framebuffer};
use bootloader::Stage16toStage32;
use serial::Serial;

mod panic;

#[no_mangle]
#[link_section = ".begin"]
fn _start(stage_to_stage: *const Stage16toStage32) {
    main(unsafe { &(*stage_to_stage) });
    panic!("Main should not return");
}

fn main(stage_to_stage: &Stage16toStage32) {
    // let video_info = &stage_to_stage.video_mode.1;
    // let mut fb = unsafe {
    //     Framebuffer::new_linear(
    //         video_info.framebuffer as *mut u32,
    //         32,
    //         video_info.height as usize,
    //         video_info.width as usize,
    //     )
    // };

    // fb.draw_rec(0, 0, fb.width(), fb.height(), Color::QUANTUM_BACKGROUND);
    let serial = Serial::probe_first(serial::baud::SerialBaud::Baud115200).unwrap();

    loop {
        serial.transmit_byte(b'H');
    }
}
