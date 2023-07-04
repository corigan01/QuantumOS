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
/// use quantum_utils::bytes::Bytes;
///
/// let size = Bytes::from(1024);
///
/// assert_eq!(format!("{}", size), "1 Kib");
/// ```
#[derive(Clone, Copy, PartialOrd, PartialEq, Default, Debug)]
pub struct Bytes(u64);

impl Bytes {
    pub const KIB: u64 = 1024;
    pub const MIB: u64 = 1024 * 1024;
    pub const GIB: u64 = 1024 * 1024 * 1024;

    /// Creates a new `Bytes` instance with zero bytes.
    pub fn new() -> Self {
        Self { 0: 0 }
    }
}

impl Display for Bytes {
    /// Formats the `Bytes` instance as a human-readable string.
    ///
    /// # Examples
    /// ```
    /// use quantum_utils::bytes::Bytes;
    ///
    /// let size = Bytes::from(2048);
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
        } else if self.0 != 1 {
            (self.0, "bytes")
        } else {
            (self.0, "byte")
        };

        write!(f, "{} {}", bytes, symb)?;

        Ok(())
    }
}

impl Add for Bytes {
    type Output = Bytes;

    fn add(self, rhs: Self) -> Self::Output {
        Bytes::from(self.0 + rhs.0)
    }
}

impl Mul for Bytes {
    type Output = Bytes;

    fn mul(self, rhs: Self) -> Self::Output {
        Bytes::from(self.0 * rhs.0)
    }
}

impl Sub for Bytes {
    type Output = Bytes;

    fn sub(self, rhs: Self) -> Self::Output {
        Bytes::from(self.0 - rhs.0)
    }
}

impl AddAssign for Bytes {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
    }
}

impl SubAssign for Bytes {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0 - rhs.0;
    }
}

impl MulAssign for Bytes {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0 * rhs.0;
    }
}

/// Converts the specified value to a `Bytes` instance.
///
/// # Examples
/// ```
/// use quantum_utils::bytes::Bytes;
///
/// let size: Bytes = 1024.into();
///
/// assert_eq!(format!("{}", size), "1 Kib");
/// ```
macro_rules! from_all_types {
    ($($t:ty)*) => ($(
        impl From<$t> for Bytes  {
            fn from(value: $t) -> Self {
                Bytes {
                    0: (value as u64)
                }
            }
        }

        impl Into<$t> for Bytes {
            fn into(self) -> $t {
                self.0 as $t
            }
        }
    )*)
}

from_all_types! {usize u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 isize}
