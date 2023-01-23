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

use core::fmt::{Debug, Formatter};
use crate::{bitset, enum_list};
use crate::bitset::BitSet;
use crate::error_utils::QuantumError;
use crate::memory::VirtualAddress;

enum_list! {
    #[repr(u64)]
    #[derive(PartialEq, Debug, Clone, Copy)]
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
        NoExecute       = 1 << 63,
        None
    }
}



#[repr(packed)]
#[derive(Clone, Copy, PartialEq, Default)]
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

    pub fn reset(&mut self) {
        self.0 = 0;
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn paste_address(&mut self, address: VirtualAddress) -> Result<(), QuantumError> {
        if !address.is_aligned() {
            return Err(QuantumError::NotAligned);
        }

        let address_shifted = address.as_u64() >> 12;

        self.0 = self.as_u64().set_bits(12..51, address_shifted);

        Ok(())

    }

    pub fn get_address(&mut self) -> Result<VirtualAddress, QuantumError> {
        let v_address = self.as_u64().get_bits(12..51) << 12;

        if v_address == 0 {
            return Err(QuantumError::NoItem);
        }

        Ok(VirtualAddress::new(v_address))
    }

    pub fn is_set(&self, flag: PageFlagOptions) -> bool {
        self.0 & (flag as u64) > 0
    }
}

impl Debug for PageFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "PageFlags(")?;
        for i in PageFlagOptions::ITEMS {
            if !self.is_set(i) { continue; }
            write!(f, "{:?}", i)?;
        }
        write!(f, ")")?;

        Ok(())
    }
}


#[cfg(test)]
pub mod test_case {
    use crate::bitset::BitSet;
    use crate::memory::paging::page_flags::{PageFlagOptions, PageFlags};
    use crate::memory::VirtualAddress;

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

        flags.reset();

        assert_eq!(flags.as_u64(), 0);
    }

    #[test_case]
    pub fn page_flags_add_address() {
        let mut page_flag = PageFlags::new();
        let address = VirtualAddress::new(0x2000);

        assert_eq!(page_flag.as_u64(), 0);

        page_flag.paste_address(address).unwrap();

        assert_eq!(page_flag.as_u64().get_bit(13), true);
        assert_eq!(page_flag.as_u64().get_bit(12), false);
        assert_eq!(page_flag.get_address().unwrap().as_u64(), 0x2000);
    }
}
