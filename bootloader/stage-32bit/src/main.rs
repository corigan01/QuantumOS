#![no_main]
#![no_std]

mod panic;

#[no_mangle]
#[link_section = ".begin"]
extern "C" fn _start() {
    main();
    panic!("Main should not return");
}

fn main() {
    unsafe { *(0xB8000 as *mut char) = '3' };
}
