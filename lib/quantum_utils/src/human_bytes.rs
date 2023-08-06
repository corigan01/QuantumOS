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

use core::fmt::{Debug, Display, Formatter};
use core::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

/// A type that represents a size in bytes, and can convert it to a human-readable string.
///
/// # Example
/// ```
/// use quantum_utils::human_bytes::HumanBytes;
///
/// let size = HumanBytes::from(1024);
///
/// assert_eq!(format!("{}", size), "1 Kib");
/// ```
#[derive(Clone, Copy, PartialOrd, PartialEq, Default, Debug)]
pub struct HumanBytes(u64);

impl HumanBytes {
    pub const KIB: u64 = 1024;
    pub const MIB: u64 = 1024 * 1024;
    pub const GIB: u64 = 1024 * 1024 * 1024;

    /// Creates a new `Bytes` instance with zero bytes.
    pub fn new() -> Self {
        Self { 0: 0 }
    }
}

impl Display for HumanBytes {
    /// Formats the `Bytes` instance as a human-readable string.
    ///
    /// # Examples
    /// ```
    /// use quantum_utils::human_bytes::HumanBytes;
    ///
    /// let size = HumanBytes::from(2048);
    ///
    /// assert_eq!(format!("{}", size), "2 Kib");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let (bytes, symb) = if self.0 >= Self::GIB {
            (self.0 / Self::GIB, "Gib")
        } else if self.0 >= Self::MIB {
            (self.0 / Self::MIB, "Mib")
        } else if self.0 >= Self::KIB {
            (self.0 / Self::KIB, "Kib")
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

impl Add for HumanBytes {
    type Output = HumanBytes;

    fn add(self, rhs: Self) -> Self::Output {
        HumanBytes::from(self.0 + rhs.0)
    }
}

impl Mul for HumanBytes {
    type Output = HumanBytes;

    fn mul(self, rhs: Self) -> Self::Output {
        HumanBytes::from(self.0 * rhs.0)
    }
}

impl Sub for HumanBytes {
    type Output = HumanBytes;

    fn sub(self, rhs: Self) -> Self::Output {
        HumanBytes::from(self.0 - rhs.0)
    }
}

impl AddAssign for HumanBytes {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
    }
}

impl SubAssign for HumanBytes {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0 - rhs.0;
    }
}

impl MulAssign for HumanBytes {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0 * rhs.0;
    }
}

/// Converts the specified value to a `Bytes` instance.
///
/// # Examples
/// ```
/// use quantum_utils::human_bytes::HumanBytes;
///
/// let size: HumanBytes = 1024.into();
///
/// assert_eq!(format!("{}", size), "1 Kib");
/// ```
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
