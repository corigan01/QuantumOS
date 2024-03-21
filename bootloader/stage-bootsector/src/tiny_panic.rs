use bioscall::video::putc;
use core::{arch::asm, panic::PanicInfo};

pub extern "C" fn fail(msg: u8) -> ! {
    putc(b':');
    putc(msg);
    putc(b'\n');
    loop {
        unsafe { asm!("hlt") };
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    fail(b'#');
}
