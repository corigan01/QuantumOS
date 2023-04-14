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

use crate::heapless_vector::HeaplessVec;
use core::fmt;
use core::fmt::{Display, Formatter, Write};
use lazy_static::lazy_static;
use spin::Mutex;

type StreamConnection = fn(&str);

#[derive(Copy, Clone, Debug)]
pub enum StreamType {
    Vga,
    Serial,
    Parallel,
    Buffered,
    Other,
}

#[derive(Copy, Clone)]
pub struct StreamConnectionBuilder {
    outlet: StreamConnection,
    stream_type: StreamType,
    bitrate: Option<usize>,
}

impl StreamConnectionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_connection(mut self, connection: StreamConnection) -> Self {
        self.outlet = connection;

        self
    }

    pub fn add_bitrate(mut self, bitrate: usize) -> Self {
        self.bitrate = Some(bitrate);

        self
    }

    pub fn add_type(mut self, t: StreamType) -> Self {
        self.stream_type = t;

        self
    }
}

impl Display for StreamConnectionBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "New Attached Debug Stream! (QuantumOS)")?;
        writeln!(f, "    StreamType: {:?}", self.stream_type)?;
        writeln!(f, "    Bitrate:    {:?}", self.bitrate)?;

        Ok(())
    }
}

impl Default for StreamConnectionBuilder {
    fn default() -> Self {
        Self {
            outlet: |_| {},
            stream_type: StreamType::Other,
            bitrate: None,
        }
    }
}

pub struct DebugStream {
    stream_connections: HeaplessVec<StreamConnectionBuilder, 4>,
}

impl DebugStream {
    pub fn new() -> Self {
        Self {
            stream_connections: HeaplessVec::new(),
        }
    }
}

impl Default for DebugStream {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Write for DebugStream {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let len = self.stream_connections.len();
        let slice = unsafe { self.stream_connections.as_mut_slice() };
        let broken_down_slice = &mut slice[..len];

        for stream in broken_down_slice {
            let outlet = stream.outlet as fn(&str);

            outlet(s);
        }

        Ok(())
    }
}

pub fn add_connection_to_global_stream(stream: StreamConnectionBuilder) {
    DEBUG_OUTPUT_STREAM
        .lock()
        .stream_connections
        .push_within_capsity(stream)
        .unwrap()
}

lazy_static! {
    static ref DEBUG_OUTPUT_STREAM: Mutex<DebugStream> = Mutex::new(DebugStream::new());
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    DEBUG_OUTPUT_STREAM.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        $crate::debug_stream::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! debug_println {
    () => ($crate::debug_print!("\n"));
    ($($arg:tt)*) => {
        $crate::debug_stream::_print(format_args!($($arg)*));
        $crate::debug_print!("\n");
    }
}
