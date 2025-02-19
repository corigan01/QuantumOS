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

#![no_std]

pub mod bytes;
pub mod consts;

/// Align `addr` to `alignment`
///
/// Pushes `addr` up until it reaches an address that satisfies `alignment`.
pub const fn align_to(addr: u64, alignment: usize) -> u64 {
    let rmd = addr % (alignment as u64);

    if rmd != 0 {
        ((alignment as u64) - rmd) + addr
    } else {
        addr
    }
}
///
/// Align `addr` range to `alignment`
///
/// Pushes `addr_start` up until it reaches an address that satisfies `alignment`.
/// Then pushes 'addr_end` down until it reaches an address that satisfies `alignment`.
pub const fn align_range_to(addr_start: u64, addr_end: u64, alignment: usize) -> (u64, u64) {
    let start = align_to(addr_start, alignment);

    (start, addr_end & !(4096 - 1))
}

/// Check the alignment of `addr` and `alignment`
pub const fn is_align_to(addr: u64, alignment: usize) -> bool {
    (addr & (alignment as u64 - 1)) == 0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_align_to() {
        assert_eq!(align_to(10, 8), 16);
        assert_eq!(align_to(16, 8), 16);
        assert_eq!(align_to(0, 8), 0);
        assert_eq!(align_to(64, 8), 64);
        assert_eq!(align_to(63, 8), 64);
    }
}
