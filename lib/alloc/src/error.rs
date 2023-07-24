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

use core::fmt::{Debug, Formatter};
use quantum_utils::bytes::Bytes;

#[derive(Clone, Copy, PartialEq)]
pub enum AllocErr {
    OutOfMemory(usize, usize),
    ImproperConfig(ImproperConfigReason),
    NotFound(usize),
    InternalErr,
    DoubleFree(usize)
}

#[derive(Clone, Copy, PartialEq)]
pub enum ImproperConfigReason {
    AlignmentInvalid(usize),
    ZeroSize,
    Smaller(usize, usize)
}


impl Debug for AllocErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            AllocErr::OutOfMemory(requested, total) => {
                write!(f,
                       "(Allocator is Out-Of-Memory) Requested {}, but only have {} available.",
                       Bytes::from(*requested), Bytes::from(*total)
                )?;
            }
            AllocErr::ImproperConfig(reason) => {
                match reason {
                    ImproperConfigReason::AlignmentInvalid(alignment) => {
                        write!(f,
                               "(Alignment {} is Invalid) Alignment must be a power of 2 and not 0.",
                                alignment
                        )?;
                    }
                    ImproperConfigReason::ZeroSize => {
                        write!(f,
                               "(Requested Zero Size Allocation) Allocations must be 1 or more bytes."
                        )?;
                    }
                    ImproperConfigReason::Smaller(old, new) => {
                        write!(f,
                               "(Does Not Fit) Your old allocation is {}, however we are trying to fit it in only {}!",
                            Bytes::from(*old),
                            Bytes::from(*new)
                        )?;
                    }
                }
            }
            AllocErr::NotFound(ptr) => {
                write!(f,
                       "(Not Found) Could not find that allocation (ptr: 0x{:x}) to free!",
                       ptr
                )?;
            }
            AllocErr::InternalErr => {
                write!(f,
                       "(Internal Allocator Error) You should assert this type of error as these are for debugging!"
                )?;
            }
            AllocErr::DoubleFree(ptr) => {
                write!(f,
                       "(Allocation Double Free) You tried to free the same memory (ptr: 0x{:x}) twice. ",
                        ptr
                )?;
            }
        }

        Ok(())
    }
}