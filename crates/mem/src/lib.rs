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

pub mod addr;
#[cfg(feature = "alloc")]
pub mod alloc;
pub mod page;
#[cfg(feature = "alloc")]
pub mod paging;
pub mod phys;
#[cfg(feature = "alloc")]
pub mod pmm;
#[cfg(feature = "alloc")]
pub mod virt2phys;
#[cfg(feature = "alloc")]
pub mod vm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryError {
    ArrayTooSmall,
    EmptySegment,
    InvalidSize,
    EntrySizeIsNegative,
    NotPageAligned,
    AlreadyUsed,
    TableNotSupported,
    PtrWasNull,
    OutOfAllocMemory,
    NotPhysicalPage,
    DoubleFree,
    NotFound,
    /// Could not handle the exception.
    DidNotHandleException,
    ParentDropped,
    InvalidPageTable,
    NotSupported,
}
