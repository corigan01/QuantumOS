use core::fmt::Write;

pub fn bios_write_char(c: char) {
    unsafe {
        core::arch::asm!("
                int 0x10
            ",
            in("ax") 0x0e00 | ((c as u16) & 0xFF),
            in("cx") 0x01,
            in("dx") 0,
        );
    }
}

pub struct BiosConsole {}

impl Write for BiosConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            match c {
                '\n' => {
                    bios_write_char('\n');
                    bios_write_char('\r');
                }
                _ => bios_write_char(c),
            }
        }

        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    (BiosConsole {}).write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! bios_print {
    ($($arg:tt)*) => {{
        $crate::console::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! bios_println {
    () => {$crate::bios_print!("\n")};
    ($($arg:tt)*) => {{
        $crate::console::_print(format_args!($($arg)*));
        $crate::bios_print!("\n");
    }};
}
