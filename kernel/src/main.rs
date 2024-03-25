#![no_main]
#![no_std]

fn _main() {}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
