/*!
```text
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
```
*/


use crate::debug_println;
use crate::memory::PAGE_SIZE;

/// # Page Map Level 4 (PML4)
/// Page map level 4 is the second highest possible page table. It stores entries of
/// `Page Directory Pointer Tables` to help abstract the memory layout of x64.
///
///
struct PageMapLevel4(u64);
struct PageDirPointerTable(u64);
struct PageDir(u64);
struct PageTable(u64);

pub fn page_align(address: u64) -> u64 {
    ( address / PAGE_SIZE as u64) * PAGE_SIZE as u64
}

impl PageMapLevel4 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Self {
            0: 0,
        }
    }
}

impl PageDirPointerTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Self {
            0: 0,
        }
    }
}

impl PageDir {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Self {
            0: 0,
        }
    }
}

impl PageTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Self {
            0: 0,
        }
    }
}


#[cfg(test)]
mod test_case {
    use crate::debug_println;
    use crate::memory::PAGE_SIZE;
    use crate::memory::paging::page_align;

    #[test_case]
    pub fn page_align_test() {
        assert_eq!(1 * PAGE_SIZE as u64, page_align(1 * PAGE_SIZE as u64 + 102));

    }
}