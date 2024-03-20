#![no_std]
#![no_main]

pub fn putc(c: u8) {
    unsafe {
        core::arch::asm!("
                mov ah, 0x0e
                int 0x10
            ",
            in("al") c
        );
    }
}

#[no_mangle]
extern "C" fn entry(disk_id: u16) {
    putc(b'H');
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
