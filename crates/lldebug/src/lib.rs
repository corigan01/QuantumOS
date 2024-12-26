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

// Re-export the macro
pub use lldebug_macro::debug_ready;
pub use lldebug_macro::make_debug;

pub mod hexdump;

/// # Output Function Type
/// The type that is required to be able to be set as an output function.
pub type OutputFn = fn(&'static str, core::fmt::Arguments);

/// # Global Output Stream Function
/// Contains the `fn` `ptr` to the function responsible for displaying debug output's arguments.
static mut GLOBAL_OUTPUT_STREAM_FUNCTION: Option<OutputFn> = None;

/// # Set Global Debug Function
/// Set the global debug `print` and `println` functions to direct their arguments to the
/// function provided.
pub fn set_global_debug_fn(output_function: OutputFn) {
    unsafe { GLOBAL_OUTPUT_STREAM_FUNCTION = Some(output_function) };
}

#[doc(hidden)]
pub fn _print(crate_name: &'static str, args: core::fmt::Arguments) {
    unsafe {
        match GLOBAL_OUTPUT_STREAM_FUNCTION {
            Some(ref output_stream) => output_stream(crate_name, args),
            _ => (),
        }
    }
}

// Re-exports for spin
pub mod sync {
    pub use spin::Mutex;

    pub struct SyncCell<T> {
        inner: ::core::cell::UnsafeCell<T>,
    }

    unsafe impl<T> Sync for SyncCell<T> {}

    impl<T> SyncCell<T> {
        pub const fn new(inner: T) -> Self {
            Self {
                inner: ::core::cell::UnsafeCell::new(inner),
            }
        }

        pub fn get(&self) -> *mut T {
            self.inner.get()
        }
    }
}

/// # Print
/// Output to global output stream.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::_print(env!("CARGO_PKG_NAME"), format_args!($($arg)*));
    }};
}

/// # `Println`
/// Output to global output stream.
#[macro_export]
macro_rules! println {
    () => {{ $crate::print!("\n") }};
    ($($arg:tt)*) => {{
        $crate::_print(env!("CARGO_PKG_NAME"), format_args!($($arg)*));
        $crate::print!("\n");
    }};
}
