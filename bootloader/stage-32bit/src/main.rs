#![no_main]
#![no_std]

use bootloader::Stage16toStage32;

mod panic;

#[no_mangle]
#[link_section = ".begin"]
extern "C" fn _start(stage_to_stage: *const Stage16toStage32) {
    main(unsafe { &(*stage_to_stage) });
    panic!("Main should not return");
}

fn main(stage_to_stage: &Stage16toStage32) {
    unsafe { *(0xB8000 as *mut char) = '3' };
}
