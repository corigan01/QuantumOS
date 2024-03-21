use crate::bios_println;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    bios_println!("{}", info);
    loop {}
}
