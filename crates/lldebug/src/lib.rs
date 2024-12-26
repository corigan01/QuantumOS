/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2024 Gavin Kellam

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

#![no_std]

use core::fmt::Write;

// Re-export the macro
pub use lldebug_macro::debug_ready;
pub use lldebug_macro::make_debug;

pub mod color;
pub mod hexdump;

// Re-exports for spin
pub mod sync {
    pub use spin::Mutex;
}

pub enum LogKind {
    Log,
    Warn,
    Error,
}

pub type OutputFn = fn(core::fmt::Arguments);

static REQUIRES_HEADER_PRINT: sync::Mutex<bool> = sync::Mutex::new(true);
static GLOBAL_PRINT_FN: sync::Mutex<Option<OutputFn>> = sync::Mutex::new(None);

fn raw_print(args: core::fmt::Arguments) {
    match GLOBAL_PRINT_FN.lock().as_ref() {
        Some(output) => output(args),
        None => (),
    }
}

pub fn set_global_debug_fn(function: OutputFn) {
    *GLOBAL_PRINT_FN.lock() = Some(function);
}

struct PrettyOutput<'a> {
    kind: LogKind,
    crate_name: &'a str,
}

impl core::fmt::Write for PrettyOutput<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        match c {
            '\n' => *REQUIRES_HEADER_PRINT.lock() = true,
            c => {
                let mut req_header = REQUIRES_HEADER_PRINT.lock();

                if *req_header {
                    *req_header = false;
                    match self.kind {
                        LogKind::Log => {
                            raw_print(format_args!("\n{}+{}", color::LOG_STYLE, color::RESET))
                        }
                        LogKind::Warn => {
                            raw_print(format_args!("\n{}-{}", color::WARN_STYLE, color::RESET))
                        }
                        LogKind::Error => {
                            raw_print(format_args!("\n{}X{}", color::ERR_STYLE, color::RESET))
                        }
                    }

                    raw_print(format_args!(
                        "{}{:<30}{} : ",
                        color::DIM_STYLE,
                        self.crate_name,
                        color::RESET
                    ));
                }

                raw_print(format_args!("{}", c));
            }
        }

        Ok(())
    }
}

#[doc(hidden)]
pub fn priv_print(kind: LogKind, crate_name: &str, args: core::fmt::Arguments) {
    let _ = PrettyOutput { kind, crate_name }.write_fmt(args);
}

/// Print a `log` message to attached console.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        $crate::priv_print(::lldebug::LogKind::Log, ::core::module_path!(), format_args!($($arg)*));
    }};
}

/// Print a `log` message to attached console with newline.
#[macro_export]
macro_rules! logln {
    () => {{ $crate::log!("\n") }};
    ($($arg:tt)*) => {{
        $crate::priv_print(::lldebug::LogKind::Log, ::core::module_path!(), format_args!($($arg)*));
        $crate::log!("\n");
    }};
}

/// Print a `warning` message to attached console.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        $crate::priv_print(::lldebug::LogKind::Warn, ::core::module_path!(), format_args!($($arg)*));
    }};
}

/// Print a `warning` message to attached console with newline.
#[macro_export]
macro_rules! warnln {
    () => {{ $crate::warn!("\n") }};
    ($($arg:tt)*) => {{
        $crate::priv_print(::lldebug::LogKind::Warn, ::core::module_path!(), format_args!($($arg)*));
        $crate::warn!("\n");
    }};
}

/// Print an `error` message to attached console.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        $crate::priv_print(::lldebug::LogKind::Error, ::core::module_path!(), format_args!($($arg)*));
    }};
}

/// Print an `error` message to attached console with newline.
#[macro_export]
macro_rules! errorln {
    () => {{ $crate::error!("\n") }};
    ($($arg:tt)*) => {{
        $crate::priv_print(::lldebug::LogKind::Error, ::core::module_path!(), format_args!($($arg)*));
        $crate::error!("\n");
    }};
}
