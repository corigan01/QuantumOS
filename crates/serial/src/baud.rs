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

/// # Serial Baud
/// Set a supported serial baud rate for serial comms.
#[derive(Clone, Copy, Debug)]
pub enum SerialBaud {
    Baud115200,
    Baud57600,
    Baud38400,
    Baud19200,
    Baud14400,
    Baud9600,
    Baud4800,
    Baud2400,
    Baud1200,
    Baud600,
    Baud300,
}

impl SerialBaud {
    /// # Get Divisor
    /// Get the divisor for serial baud register's clock. This will
    /// get the divisor setting needed to get to one of the set baud
    /// rates.
    pub const fn get_divisor(self) -> u16 {
        match self {
            Self::Baud115200 => 1,
            Self::Baud57600 => 2,
            Self::Baud38400 => 3,
            Self::Baud19200 => 6,
            Self::Baud14400 => 8,
            Self::Baud9600 => 12,
            Self::Baud4800 => 24,
            Self::Baud2400 => 48,
            Self::Baud1200 => 96,
            Self::Baud600 => 192,
            Self::Baud300 => 348,
        }
    }
}
