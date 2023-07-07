/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
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

use crate::debug::heapless_stream_collections::HeaplessStreamCollections;
use crate::debug::stream_connection::StreamConnection;
use over_stacked::heapless_vector::HeaplessVecErr;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;

pub mod heapless_stream_collections;
pub mod stream_connection;

pub type SimpleStreamFunction = fn(&str);

pub fn add_connection_to_global_stream(stream: StreamConnection) -> Result<(), HeaplessVecErr> {
    let does_stream_want_welcome = !stream.ignore_welcome;
    let stream_info_copy = stream.info;

    let status = DEBUG_OUTPUT_STREAM
        .lock()
        .stream_connections
        .push_within_capacity(stream);

    if does_stream_want_welcome {
        crate::debug_println!(
            "\n{}\n'{}' was just added! All project log/debug messages will be shown here.",
            "---------------------------------------------------------------------------",
            stream_info_copy.connection_name);
    }

    status
}

lazy_static! {
    static ref DEBUG_OUTPUT_STREAM: Mutex<HeaplessStreamCollections> =
        Mutex::new(HeaplessStreamCollections::new());
}

pub fn set_panic() {
    DEBUG_OUTPUT_STREAM.lock().is_panic = true;
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    DEBUG_OUTPUT_STREAM.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        $crate::debug::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! debug_println {
    () => ($crate::debug_print!("\n"));
    ($($arg:tt)*) => {
        $crate::debug::_print(format_args!($($arg)*));
        $crate::debug_print!("\n");
    }
}
