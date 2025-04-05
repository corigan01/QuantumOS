/*
  ____                 __               __  __
 / __ \__ _____ ____  / /___ ____ _    / / / /__ ___ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /_/ (_-</ -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/  \____/___/\__/_/
  Part of the Quantum OS Kernel

Copyright 2025 Gavin Kellam

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

use core::fmt::Error;
use core::fmt::Write;
use vera_portal::sys_client::debug_msg;

#[doc(hidden)]
pub use lignan::set_global_debug_fn;

/// Quantum OS's 'kernel' debug output.
///
/// This is used in the `dbug!(...)` and `dbugln!(...)` macros
/// to give a 'println!()' like enviroment that outputs into the
/// kernel's debug device (useally the serial port).
pub struct DebugOut {}

impl core::fmt::Write for DebugOut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        debug_msg(s).map_err(|_| Error {})
    }
}

#[doc(hidden)]
pub fn priv_print(args: core::fmt::Arguments) {
    let _ = DebugOut {}.write_fmt(args);
}

/// Quantum OS's 'kernel' debug formatting macro.
///
/// Outputs like 'println', but instead of going to StdOut, this
/// macro prints to the kernel's debug output (useally a serial port).
#[macro_export]
macro_rules! dbug {
    ($($arg:tt)*) => {{
        $crate::debug::priv_print(format_args!($($arg)*));
    }};
}

/// Quantum OS's 'kernel' debug formatting macro.
///
/// Outputs like 'println', but instead of going to StdOut, this
/// macro prints to the kernel's debug output (useally a serial port).
#[macro_export]
macro_rules! dbugln {
    () => {{ $crate::debug::dbug!("\n") }};
    ($($arg:tt)*) => {{
        $crate::debug::priv_print(format_args!($($arg)*));
        $crate::dbug!("\n");
    }};
}
