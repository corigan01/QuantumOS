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

use core::fmt::{Display, Formatter};

const ROWS_TO_PRINT: usize = 10;
const BYTES_PER_ROW: usize = 2;

pub struct HexPrinter<'a> {
    data: &'a [u8]
}

impl<'a> Display for HexPrinter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("\nDumping Array of size {}!\n| ", self.data.len()))?;
        let mut line_string = [0u8; ROWS_TO_PRINT];

        for index in 0..self.data.len().checked_next_multiple_of(ROWS_TO_PRINT).unwrap_or(0) {
            let byte = self.data.get(index).unwrap_or(&0);

            if index > self.data.len() {
                f.write_fmt(format_args!("--"))?;
            } else {
                f.write_fmt(format_args!("{:02X}", byte))?;
            }

            line_string[index % ROWS_TO_PRINT] = *byte;

            if (index + 1) % 2 == 0 {
                f.write_fmt(format_args!(" "))?;
            }
            if (index + 1) % (ROWS_TO_PRINT * BYTES_PER_ROW) == 0 {
                f.write_fmt(format_args!("| "))?;

                for char_byte in line_string.iter() {
                    if char_byte.is_ascii_alphanumeric() || char_byte.is_ascii_alphabetic() {
                        f.write_fmt(format_args!("{}", *char_byte as char))?;
                    } else {
                        f.write_fmt(format_args!("."))?;
                    }
                }

                f.write_fmt(format_args!("\n"))?;

                if index < self.data.len() - 1 {
                    f.write_fmt(format_args!("| "))?;
                } else {
                    f.write_fmt(format_args!("\n"))?;
                }
            }
        }

        Ok(())
    }
}

pub trait HexPrint {
    fn hex_print(&self) -> HexPrinter;
}

impl HexPrint for &[u8] {
    fn hex_print(&self) -> HexPrinter {
        HexPrinter {
            data: self
        }
    }
}

impl<const SIZE: usize> HexPrint for [u8; SIZE] {
    fn hex_print(&self) -> HexPrinter {
        HexPrinter {
            data: self
        }
    }
}