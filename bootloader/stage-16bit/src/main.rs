#![no_std]
#![no_main]

mod panic;

#[no_mangle]
#[link_section = ".begin"]
extern "C" fn entry(disk_id: u16) {
    for c in b"test" {
        panic::putc(*c);
    }
    // panic!("Panic!");

    loop {}
}
