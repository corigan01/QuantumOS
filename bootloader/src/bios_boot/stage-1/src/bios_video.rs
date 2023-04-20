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

use quantum_lib::x86_64::bios_call::BiosCall;
use core::fmt;

pub struct BiosTextMode {}

impl BiosTextMode {
    pub fn new() -> Self {
        Self {}
    }

    unsafe fn print_int_char(c: u8) {
        BiosCall::new()
            .bit16_call()
            .display_char(c as char, 0x0000)
            .ignore_error()
    }

    pub fn print_int_bytes(str: &[u8]) {
        for i in str {
            match *i {
                b'\n' => unsafe {
                    Self::print_int_char(0xd);
                    Self::print_int_char(0xa);
                },
                c if c.is_ascii() => unsafe {
                    Self::print_int_char(c);
                },
                _ => {
                    unsafe { Self::print_int_char(b'?') };
                    break;
                }
            }
        }
    }
}

impl fmt::Write for BiosTextMode {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Self::print_int_bytes(s.as_bytes());

        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    BiosTextMode::new().write_fmt(args).unwrap();
}

/// Prints to the host through the bios int.
#[cfg(debug)]
#[macro_export]
macro_rules! bios_print {
    ($($arg:tt)*) => {
        $crate::bios_video::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the bios int, appending a newline.
#[cfg(debug)]
#[macro_export]
macro_rules! bios_println {
    () => ($crate::bios_print!("\n"));
    ($($arg:tt)*) => {
        $crate::bios_video::_print(format_args!($($arg)*));
        $crate::bios_print!("\n");
    }
}
