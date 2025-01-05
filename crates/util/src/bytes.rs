/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

/// A type that represents a size in bytes, and can convert it to a human-readable string.
///
/// # Example
/// ```
/// use util::bytes::HumanBytes;
///
/// let size = HumanBytes::from(1024);
///
/// assert_eq!(format!("{}", size), "1 Kib");
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, PartialOrd, PartialEq, Default, Debug, Eq, Ord)]
pub struct HumanBytes(pub u64);

impl HumanBytes {
    const KIB_U64: u64 = 1024;
    const MIB_U64: u64 = 1024 * 1024;
    const GIB_U64: u64 = 1024 * 1024 * 1024;

    /// Make a new `HumanBytes` with zero bytes.
    pub fn zero() -> Self {
        Self(0)
    }

    /// Make a new `HumanBytes` with given `bytes`.
    pub fn new(bytes: u64) -> Self {
        Self(bytes)
    }
}

pub trait FromKib<Type>
where
    Type: Into<u64>,
{
    fn from_kib(kib: Type) -> HumanBytes {
        let value: u64 = kib.into();

        HumanBytes(value * HumanBytes::KIB_U64)
    }
}

pub trait FromMib<Type>
where
    Type: Into<u64>,
{
    fn from_kib(kib: Type) -> HumanBytes {
        let value: u64 = kib.into();

        HumanBytes(value * HumanBytes::MIB_U64)
    }
}

pub trait FromGib<Type>
where
    Type: Into<u64>,
{
    fn from_kib(kib: Type) -> HumanBytes {
        let value: u64 = kib.into();

        HumanBytes(value * HumanBytes::GIB_U64)
    }
}

impl<T: Into<u64>> FromKib<T> for HumanBytes {}
impl<T: Into<u64>> FromMib<T> for HumanBytes {}
impl<T: Into<u64>> FromGib<T> for HumanBytes {}

impl Display for HumanBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let (bytes, symb) = if self.0 >= Self::GIB_U64 {
            (self.0 / Self::GIB_U64, "Gib")
        } else if self.0 >= Self::MIB_U64 {
            (self.0 / Self::MIB_U64, "Mib")
        } else if self.0 >= Self::KIB_U64 {
            (self.0 / Self::KIB_U64, "Kib")
        } else {
            (self.0, "bytes")
        };

        write!(f, "{} {}", bytes, symb)?;

        let Some(width) = f.width() else {
            return Ok(());
        };

        let digit_chars = match bytes {
            i if i < 10 => 1,
            i if i < 100 => 2,
            i if i < 1000 => 3,
            i if i < 10000 => 4,
            _ => 0,
        };
        let symb_chars = symb.chars().count();

        let total_chars = digit_chars + symb_chars + 1;
        if total_chars > width {
            return Ok(());
        }

        let padding = width - total_chars;

        for _ in 0..padding {
            write!(f, " ")?;
        }

        Ok(())
    }
}

macro_rules! from_all_types {
    ($($t:ty)*) => ($(
        impl From<$t> for HumanBytes  {
            fn from(value: $t) -> Self {
                HumanBytes {
                    0: (value as u64)
                }
            }
        }

        impl Into<$t> for HumanBytes {
            fn into(self) -> $t {
                self.0 as $t
            }
        }
    )*)
}

from_all_types! {usize u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 isize}
