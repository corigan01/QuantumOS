/*
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
furnished to do so, subject to the following conditions

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

use crate::bitset;

#[repr(u64)]
#[derive(PartialEq)]
pub enum PageFlagOptions {
    Present         = 1 << 0,
    Writable        = 1 << 1,
    UserAccessible  = 1 << 2,
    WriteThrough    = 1 << 3,
    NoCache         = 1 << 4,
    Accessed        = 1 << 5,
    Dirty           = 1 << 6,
    HugePage        = 1 << 7,
    Global          = 1 << 8,
    UnusedBit9      = 1 << 9,
    UnusedBit10     = 1 << 10,
    UnusedBit11     = 1 << 11,
    UnusedBit52     = 1 << 52,
    UnusedBit53     = 1 << 53,
    UnusedBit54     = 1 << 54,
    UnusedBit55     = 1 << 55,
    UnusedBit56     = 1 << 56,
    UnusedBit57     = 1 << 57,
    UnusedBit58     = 1 << 58,
    UnusedBit59     = 1 << 59,
    UnusedBit60     = 1 << 60,
    UnusedBit61     = 1 << 61,
    UnusedBit62     = 1 << 62,
    NoExecute       = 1 << 63
}

#[repr(packed)]
pub struct PageFlags(u64);

impl PageFlags {
    pub fn new() -> Self {
        Self {
            0: 0
        }
    }

    pub fn enable(&mut self, flag: PageFlagOptions) -> &mut Self {
        self.0 |= flag as u64;

        self
    }

    pub fn disable(&mut self, flag: PageFlagOptions) -> &mut Self {
        self.0 = self.0 & !( flag as u64 );

        self
    }

    pub fn disable_all(&mut self) {
        self.0 = 0;
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

}


#[cfg(test)]
pub mod test_case {
    use crate::memory::paging::page_flags::{PageFlagOptions, PageFlags};

    #[test_case]
    pub fn page_flags_enable_disable_test() {
        let mut flags = PageFlags::new();

        flags.enable(PageFlagOptions::Writable);

        assert_eq!(flags.as_u64(), PageFlagOptions::Writable as u64);

        flags.disable(PageFlagOptions::Writable);
        flags.enable(PageFlagOptions::Present);

        assert_eq!(flags.as_u64(), PageFlagOptions::Present as u64);

        flags.disable(PageFlagOptions::Present);
        flags.enable(PageFlagOptions::Dirty);

        assert_eq!(flags.as_u64(), PageFlagOptions::Dirty as u64);

        flags.disable(PageFlagOptions::Dirty);

        assert_eq!(flags.as_u64(), 0);

        flags.enable(PageFlagOptions::HugePage);
        flags.enable(PageFlagOptions::Present);
        flags.enable(PageFlagOptions::UserAccessible);

        flags.disable_all();

        assert_eq!(flags.as_u64(), 0);
    }
}
