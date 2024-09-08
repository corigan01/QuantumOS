#![no_main]
#![no_std]

use bootgfx::{Color, Framebuffer};
use bootloader::Stage16toStage32;

mod panic;

#[no_mangle]
#[link_section = ".begin"]
extern "C" fn _start(stage_to_stage: *const Stage16toStage32) {
    main(unsafe { &(*stage_to_stage) });
    panic!("Main should not return");
}

fn main(stage_to_stage: &Stage16toStage32) {
    let video_info = &stage_to_stage.video_mode.1;
    let mut fb = unsafe {
        Framebuffer::new_linear(
            video_info.framebuffer as *mut u32,
            32,
            video_info.height as usize,
            video_info.width as usize,
        )
    };

    fb.draw_rec(10, 10, 100, 100, Color::QUANTUM_BACKGROUND);
    loop {}
}
