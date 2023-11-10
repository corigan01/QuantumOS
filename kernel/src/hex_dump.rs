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

use core::fmt::{Display, Formatter, Write};

const ROWS_TO_PRINT: usize = 10;
const BYTES_PER_ROW: usize = 2;

pub struct HexPrinter<'a> {
    data: &'a [u8],
}

impl<'a> HexPrinter<'a> {
    const INBUILT_BUFFER_ARRAY: [u8; ROWS_TO_PRINT] = [0; ROWS_TO_PRINT];
}

impl<'a> Display for HexPrinter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_char('\n')?;
        self.data.chunks(ROWS_TO_PRINT).try_for_each(|chunk_print| {
            f.write_str(" | ")?;
            chunk_print.chunks(BYTES_PER_ROW).try_for_each(|value| {
                value
                    .iter()
                    .try_for_each(|byte| f.write_fmt(format_args!("{:02x}", byte)))?;
                f.write_str(" ")
            })?;

            Self::INBUILT_BUFFER_ARRAY[..(ROWS_TO_PRINT - chunk_print.len())]
                .chunks(BYTES_PER_ROW)
                .try_for_each(|_| f.write_str("     "))?;

            f.write_str(" | ")?;
            chunk_print.iter().try_for_each(|val| {
                f.write_char(match val {
                    0 => '.',
                    v if v.is_ascii_alphanumeric() => *v as char,
                    _ => '_',
                })
            })?;

            Self::INBUILT_BUFFER_ARRAY[..(ROWS_TO_PRINT - chunk_print.len())]
                .iter()
                .try_for_each(|_| f.write_char(' '))?;

            f.write_str(" |\n")
        })
    }
}

pub trait HexPrint {
    fn hex_print(&self) -> HexPrinter;
}

impl HexPrint for &[u8] {
    fn hex_print(&self) -> HexPrinter {
        HexPrinter { data: self }
    }
}

impl<const SIZE: usize> HexPrint for [u8; SIZE] {
    fn hex_print(&self) -> HexPrinter {
        HexPrinter { data: self }
    }
}

