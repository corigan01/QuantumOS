/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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
use core::fmt::Write;
use spin::Mutex;
use lazy_static::lazy_static;
use owo_colors::OwoColorize;
use crate::{serial_println, serial_print };

pub type OutputStream = fn(&str);

pub struct StreamInfo {
    pub output_stream: Option<OutputStream>,
    pub name: Option<&'static str>,
    pub speed: Option<u64>,
}

impl StreamInfo {
    pub fn new() -> Self {
        Self {
            name: None,
            speed: None,
            output_stream: None,
        }
    }

    pub fn fill_from(&mut self, input_pram: Self) {
        self.name = input_pram.name;
        self.speed = input_pram.speed;
        self.output_stream = input_pram.output_stream;
    }

    pub fn write_string(&self, s: &str) {
        if let Some(stream) =  self.output_stream {
            for i in s.bytes() {

            }
            stream(s);
        }
    }
}

impl fmt::Write for StreamInfo {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    static ref DEBUG_OUTPUT_STREAM: Mutex<StreamInfo> = {
        Mutex::new(StreamInfo::new())
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    DEBUG_OUTPUT_STREAM.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        $crate::debug_output::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! debug_println {
    () => ($crate::debug_print!("\n"));
    ($fmt:expr) => ($crate::debug_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::debug_print!(
        concat!($fmt, "\n"), $($arg)*));
}

pub fn set_stream(stream_info: StreamInfo) {
    let (output_some, name_option, speed_option) = {
        let mut global_stream = DEBUG_OUTPUT_STREAM.lock();
        global_stream.fill_from(stream_info);

        (global_stream.output_stream.is_some(), global_stream.name, global_stream.speed)
    };

    if output_some {
        debug_println!("\n\n{}", "Quantum is using this stream for debug information!".bright_green().bold());
        

        if let Some(name) = name_option {
            debug_println!("\t- {}: {}", "Debug Output Name".bright_blue(), name.bright_green().bold());
        }
        if let Some(speed) = speed_option {
            debug_println!("\t- {}: {}", "Speed Information".bright_blue(), speed.bright_green().bold());
        }
    }
}