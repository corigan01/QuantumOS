#![no_main]
#![no_std]

mod panic;

#[no_mangle]
#[link_section = ".begin"]
extern "C" fn _start() {
    loop {}
    panic!("Main should not return");
}

fn main() {
    unsafe { *(0x8B000 as *mut char) = '3' };
}
