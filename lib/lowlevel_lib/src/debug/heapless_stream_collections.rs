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

use crate::debug::stream_connection::StreamConnection;
use over_stacked::heapless_vector::HeaplessVec;
use owo_colors::OwoColorize;

pub struct HeaplessStreamCollections {
    pub(crate) stream_connections: HeaplessVec<StreamConnection, 4>,
    pub(crate) is_panic: bool,
}

impl HeaplessStreamCollections {
    pub fn new() -> Self {
        Self {
            stream_connections: HeaplessVec::new(),
            is_panic: false,
        }
    }
}

impl Default for HeaplessStreamCollections {
    fn default() -> Self {
        Self::new()
    }
}

impl ::core::fmt::Write for HeaplessStreamCollections {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for stream in self.stream_connections.mut_iter() {
            if let Some(outlet) = &mut stream.outlet {
                for char in s.chars() {
                    if char == '\n' {
                        if self.is_panic {
                            write!(
                                outlet,
                                "\n[{}: {}]: ",
                                stream.info.who_using.dimmed().bold(),
                                "PANIC".red().bold().blink()
                            )?;
                            continue;
                        }

                        write!(
                            outlet,
                            "\n[{} -> {}]: ",
                            stream.info.who_using.dimmed().bold(),
                            stream.info.connection_name.dimmed().bold()
                        )?;

                        continue;
                    }

                    write!(outlet, "{}", char)?;
                }

                continue;
            }

            if let Some(outlet) = stream.simple_outlet {
                let func = outlet as fn(&str);
                func(s);

                continue;
            }
        }

        Ok(())
    }
}
