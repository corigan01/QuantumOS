/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
  Part of the Quantum OS Project

Copyright 2023 Gavin Kellam

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/


use core::fmt;
use crate::bios_ints::{BiosInt, TextModeColor};
use crate::console::{GlobalPrint};

pub struct BiosTextMode {
    background_color: TextModeColor,
    foreground_color: TextModeColor,
}

impl BiosTextMode {
    pub fn new() -> Self {
        Self {
            background_color: TextModeColor::Black,
            foreground_color: TextModeColor::White
        }
    }

    unsafe fn print_int_char(&self, c: u8) {
        BiosInt::write_character(
            c,
            0,
            self.background_color,
            self.foreground_color)

            .execute_interrupt();
    }


    fn print_int_bytes(&self, str: &[u8]) {
        for i in str {
            match *i  {
                b'\n' => unsafe { self.print_int_char(0xd); self.print_int_char(0xa); },
                c if c.is_ascii() => unsafe { self.print_int_char(c); },

                _ => unsafe { self.print_int_char(b'?'); }
            }
        }
    }

    fn print_int_string(&self, str: &str) {
        self.print_int_bytes(str.as_bytes());
    }
}


impl GlobalPrint for BiosTextMode {
    fn print_str(str: &str) {
        BiosTextMode::new().print_int_string(str);
    }
    fn print_bytes(bytes: &[u8]) { BiosTextMode::new().print_int_bytes(bytes); }
}

impl fmt::Write for BiosTextMode {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print_int_string(s);

        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    BiosTextMode::new().write_fmt(args).unwrap();
}

/// Prints to the host through the bios int.
#[macro_export]
macro_rules! bios_print {
    ($($arg:tt)*) => {
        $crate::bios_video::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the bios int, appending a newline.
#[macro_export]
macro_rules! bios_println {
    () => ($crate::bios_print!("\n"));
    ($fmt:expr) => ($crate::bios_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::bios_print!(
        concat!($fmt, "\n"), $($arg)*));
}