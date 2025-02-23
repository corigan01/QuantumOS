/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

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

// use core::cell::{Cell, UnsafeCell};
// use core::fmt::Debug;
use core::cell::RefCell;

pub type ThreadCell<T> = RefCell<T>;

// /// A cell that promises the only accessor will be the thread that owns it.
// pub struct ThreadCell<T: ?Sized> {
//     borrows: Cell<usize>,
//     inner: UnsafeCell<T>,
// }

// impl<T: ?Sized + Debug> Debug for ThreadCell<T> {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         todo!()
//     }
// }

// impl<T> ThreadCell<T> {
//     /// Create a new thread cell
//     pub fn new(value: T) -> Self {
//         Self {
//             borrows: Cell::new(0),
//             inner: UnsafeCell::new(value),
//         }
//     }
// }

// pub struct ThreadCellRef<'a, T: ?Sized> {
//     borrows: &'a Cell<usize>,
//     ptr: *const T,
// }

// pub struct ThreadCellMut<'a, T: ?Sized> {
//     borrows: &'a Cell<usize>,
//     ptr: *mut T,
// }
