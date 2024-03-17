use core::{arch::asm, panic::PanicInfo};

pub fn putc(c: u8) {
    unsafe {
        asm!("
                mov ah, 0x0e
                int 0x10
            ",
            in("al") c
        );
    }
}

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
