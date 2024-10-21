/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2024 Gavin Kellam

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

#[derive(Clone, Copy)]
pub struct PageDirectryEntry(u64);

macro_rules! impl_bit {
    {$name:ident, $bit:literal $(#[$meta:meta])*} => {
        $(#[$meta])*
        /// # Important Considerations
        /// This function is `const` and will be best used when setup in 'chains' at compile time.
        ///
        /// For Example, one might:
        /// ```rust
        /// use arch::paging::PageDirectryEntry;
        ///
        /// let entry = PageDirectryEntry::empty()
        ///     .present(true)
        ///     .read_write(true)
        ///     .user_access(true);
        /// ```
        /// Here we are building a `PageDirectryEntry` with the `P`, `R/W`, and `U/S` bits set. The
        /// bit-field in `entry` will correspond to this change (should be compiled in).
        ///
        /// # Safety
        /// This is not 'unsafe however, its not fully 'safe' either. When loading the page
        /// tables themselves, one must understand and verify that all page tables are
        /// loaded correctly. Each entry in the page table isn't unsafe by itself,
        /// however, when loaded into the system it becomes unsafe.
        ///
        /// It would be a good idea to verify that this 'bit' or option does exactly what you
        /// intend it to do before loading it. Page tables can cause the entire system to become
        /// unstable if mapped wrong -- **this is very important.**
        pub const fn $name(&mut self, value: bool) -> Self {
            if value {
                self.0 |= 1 << $bit;
            } else {
                self.0 &= !(1 << $bit);
            }

            *self
        }
    };
}

impl PageDirectryEntry {
    pub const fn empty() -> Self {
        Self(0)
    }

    impl_bit! {present, 0
        /// Set the 'present' page table bit
        ///
        /// This bit determines if the page entry is 'active' or 'inactive The
        /// CPU on x86 systems will always generate a #PF (Page Fault Exception)
        /// when this bit is not set regardless of the rest of the page table's
        /// state.
        ///
    }
    impl_bit! {read_write, 1
        /// Set the 'read/write' page table bit
        ///
        /// This bit determines if the page entry is able to be written to. In
        /// its default state, the page is `RO` (Read Only). When setting this
        /// bit to `true` it will allow access to `RW` (Read/Write) to the memory
        /// pointed to by this page entry.
    }
    impl_bit! {user_access, 2
        /// Set the 'superuser/user' page table bit.
        ///
        /// This bit determines if the page table is accessable from userspace.
        /// In its default state, page entries are **not** accessable from userspace,
        /// and will cause a #PF (Page Fault Exception).
    }
}
