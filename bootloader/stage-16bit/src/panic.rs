use core::{fmt::Write, panic::PanicInfo};

pub fn putc(c: u8) {
    unsafe {
        core::arch::asm!("
                int 0x10
            ",
            in("ax") 0x0e00 | c as u16,
            in("cx") 0x01,
            in("dx") 0,
        );
    }
}

struct BiosPrinter {}

impl Write for BiosPrinter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            match byte {
                b'\n' => {
                    putc(byte);
                    putc(b'\r');
                }
                b => putc(byte),
            }
        }
        Ok(())
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    BiosPrinter::write_fmt(
        &mut BiosPrinter {},
        format_args!("\n\n----- PANIC -----\n{}", info),
    )
    .unwrap();
    loop {}
}
