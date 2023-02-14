#![no_main]
#![no_std]

use core::panic::PanicInfo;
use core::arch::{asm, global_asm};


global_asm!(include_str!("init.s"));



#[no_mangle]
extern "C" fn bit16_entry() {
    let disk: u16;
    unsafe {
        asm!(
            "nop",
            out("dx") disk
        );
    };

    enter_rust(disk);
    panic!("Stage should not return!");
}

fn enter_rust(disk: u16) {

}



#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {};
}