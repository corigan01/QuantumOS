#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

mod vga_buffer;

mod vga;

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga_println!("{}", info);
    loop {}
}



#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga_println!("Quantum OS v0.1.0");
    vga_println!("---------------------");

    vga_println!();
    vga_println!("[CPU] INIT");


    loop {}
}