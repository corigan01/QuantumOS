#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

//mod vga;

use core::panic::PanicInfo;
use bootloader::boot_info::BootInfo;
use bootloader::entry_point;

entry_point!(main);

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    //vga_println!("{}", info);
    loop {}
}



fn main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        for byte in framebuffer.buffer_mut() {
            *byte = 0xF0;
        }
        for byte in framebuffer.buffer_mut() {
            *byte = 0x00;
        }

        let buffer = framebuffer.buffer_mut();

        buffer[1] = 0xF0;
    }



    loop {}
}