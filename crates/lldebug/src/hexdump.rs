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

use core::fmt::{Display, Formatter, Write};

const ROWS_TO_PRINT: usize = 16;
const BYTES_PER_ROW: usize = 2;

const INCLUDE_HEADER_AND_FOOTER: bool = true;

pub struct HexDump<'a> {
    data: &'a [u8],
}

impl<'a> HexDump<'a> {
    const INBUILT_BUFFER_ARRAY: [u8; ROWS_TO_PRINT] = [0; ROWS_TO_PRINT];
}

impl<'a> Display for HexDump<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_char('\n')?;

        if INCLUDE_HEADER_AND_FOOTER {
            f.write_fmt(format_args!(" + {:9} +", self.data.len()))?;

            for _ in 0..(ROWS_TO_PRINT / BYTES_PER_ROW) * 5 + 1 {
                f.write_char('-')?;
            }
            f.write_char('+')?;
            for _ in 0..ROWS_TO_PRINT + 2 {
                f.write_char('-')?;
            }

            f.write_str("+\n")?;
        }

        self.data
            .chunks(ROWS_TO_PRINT)
            .enumerate()
            .try_for_each(|(enumerate, chunk_print)| {
                f.write_fmt(format_args!(" | {:09x} | ", enumerate * ROWS_TO_PRINT))?;
                chunk_print.chunks(BYTES_PER_ROW).try_for_each(|value| {
                    value
                        .iter()
                        .try_for_each(|byte| f.write_fmt(format_args!("{:02x}", byte)))?;
                    f.write_str(" ")
                })?;

                Self::INBUILT_BUFFER_ARRAY[..(ROWS_TO_PRINT - chunk_print.len())]
                    .chunks(BYTES_PER_ROW)
                    .try_for_each(|_| f.write_str("     "))?;

                f.write_str("| ")?;
                chunk_print.iter().try_for_each(|val| {
                    f.write_char(match val {
                        0 => '.',
                        b' ' => ' ',
                        v if v.is_ascii_alphanumeric() || v.is_ascii_punctuation() => *v as char,
                        _ => '.',
                    })
                })?;

                Self::INBUILT_BUFFER_ARRAY[..(ROWS_TO_PRINT - chunk_print.len())]
                    .iter()
                    .try_for_each(|_| f.write_char(' '))?;

                f.write_str(" |\n")
            })?;

        if INCLUDE_HEADER_AND_FOOTER {
            f.write_str(" +-----------+")?;

            for _ in 0..(ROWS_TO_PRINT / BYTES_PER_ROW) * 5 + 1 {
                f.write_char('-')?;
            }
            f.write_char('+')?;
            for _ in 0..ROWS_TO_PRINT + 2 {
                f.write_char('-')?;
            }

            f.write_str("+\n")?;
        }

        Ok(())
    }
}

pub trait HexPrint {
    fn hexdump(&self) -> HexDump;
}

impl HexPrint for &[u8] {
    fn hexdump(&self) -> HexDump {
        HexDump { data: self }
    }
}

impl HexPrint for &mut [u8] {
    fn hexdump(&self) -> HexDump {
        HexDump { data: self }
    }
}

impl<const SIZE: usize> HexPrint for [u8; SIZE] {
    fn hexdump(&self) -> HexDump {
        HexDump { data: self }
    }
}
