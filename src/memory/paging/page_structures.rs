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
use crate::error_utils::QuantumError;
use crate::memory::paging::page_flags::{PageFlagOptions, PageFlags};
use crate::memory::VirtualAddress;

struct PageMapLevel4(PageFlags);
struct PageDirPointerTable(PageFlags);
struct PageDir(PageFlags);
struct PageTable(PageFlags);

impl PageMapLevel4 {
    const INVALID_OPTIONS: [PageFlagOptions; 1] = [PageFlagOptions::Present];

    pub fn new() -> Self {
        Self::valid_default()
    }

    pub fn valid_default() -> Self {
        Self {
            0: PageFlags::new()
        }
    }

    pub fn delete_all_flags(&mut self) {
        self.0.disable_all();
    }

    pub fn enable_flag(&mut self, option: PageFlagOptions) -> Result<(), QuantumError> {

        // Make sure that option is valid for this paging structure
        for i in Self::INVALID_OPTIONS {
            if option == i {
                return Err(QuantumError::InvalidOption);
            }
        }

        // Enable the page
        self.0.enable(option);

        Ok(())
    }

    pub fn set_address(&self, address: VirtualAddress) {


    }


    pub fn compile_out(&self) -> u64 {
        // TODO: Implement checking to make sure that the structure is valid

        self.0.as_u64()
    }
}

impl PageDirPointerTable {
    pub fn new() -> Self {
        Self::valid_default()
    }

    pub fn valid_default() -> Self {
        Self {
            0: PageFlags::new()
        }
    }
}

impl PageDir {
    pub fn new() -> Self {
        Self::valid_default()
    }

    pub fn valid_default() -> Self {
        Self {
            0: PageFlags::new()
        }
    }
}

impl PageTable {
    pub fn new() -> Self {
        Self::valid_default()
    }

    pub fn valid_default() -> Self {
        Self {
            0: PageFlags::new()
        }
    }
}