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

#![no_std]
#![feature(test)]
#![feature(receiver_trait)]
#![feature(unsize)]
#![feature(dispatch_from_dyn)]
#![feature(coerce_unsized)]

pub mod heap;
pub mod memory_layout;
pub mod usable_region;

// std(like) libs for heap allocation
pub mod bitfield;
pub mod boxed;
pub mod string;
pub mod vec;

pub mod borrowed_buf;
pub mod circular_buffer;
pub mod error;
pub mod buffer;

pub use error::*;

#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut new_string = qk_alloc::string::String::new();
        write!(&mut new_string, "{}", format_args!($($arg)*)).unwrap();
        new_string
    }}
}

#[macro_export]
macro_rules! vec {
    () => (
        $crate::vec::Vec::new()
    );
    ($elem:expr; $n:expr) => (
        $crate::vec::from_elem($elem, $n)
    );
    ($($x:expr),*) => (
        $crate::slice::into_vec($crate::boxed::Box::new([$($x),*]))
    );
    ($($x:expr,)*) => (vec![$($x),*])
}
