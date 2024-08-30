use bios::video;
use core::fmt::Write;

pub struct BiosConsole {}

impl Write for BiosConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            match c {
                '\n' => {
                    video::print_char('\n');
                    video::print_char('\r');
                }
                _ => video::print_char(c),
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
