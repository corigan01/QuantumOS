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

macro_rules! impl_bit {
    // TODO: Make a proc macro for doing this bit stuff in a more general way for more things to use it
    {$name:ident, $get_name: ident, $bit:literal $(#[$meta:meta])*} => {
        $(#[$meta])*
        /// # Important Considerations
        /// This function is `const` and will be best used when setup in 'chains' at compile time.
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

        // FIXME: Fix these docs
        /// # The 'get' version of the docs below.
        ///
        /// ---
        $(#[$meta])*
        /// # Important Considerations
        /// This function is `const` and will be best used when setup in 'chains' at compile time.
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
        pub const fn $get_name(&self) -> bool {
            ((self.0 >> $bit) & 0b1) != 0
        }
    };
}

macro_rules! impl_entry {
    ($($(#[$meta:meta])* $entry_name: ident),*) => {
        $(
        $(#[$meta])*
        /// # How to use?
        /// For Example, one might:
        /// ```rust
        /// use arch::paging::PageDirectryEntry;
        ///
        /// let entry = PageDirectryEntry::empty()
        ///     .present(true)
        ///     .read_write(true)
        ///     .user_access(true);
        /// ```
        ///
        /// Here we are building a `PageDirectryEntry` with the `P`, `R/W`, and `U/S` bits set. The
        /// bit-field in `entry` will correspond to this change (should be compiled in).
        ///
        /// # Safety
        /// This is not 'unsafe however, its not fully 'safe' either. When loading the page
        /// tables themselves, one must understand and verify that all page tables are
        /// loaded correctly. Each entry in the page table isn't unsafe by itself,
        /// however, when loaded into the system it becomes unsafe.
        ///
        /// It would be a good idea to verify that all 'bit' or options set in this entry  does exactly
        /// what you intend it to do before loading it. Page tables can cause the entire system to become
        /// unstable if mapped wrong -- **this is very important.**
        #[derive(Clone, Copy)]
        pub struct $entry_name(u64);

        impl $entry_name {
            /// Make a completly blank page table entry.
            ///
            /// None of the bits are set.
            pub const fn empty() -> Self {
                Self(0)
            }

            impl_bit! {present, get_present, 0
                /// Set the 'present' page table bit
                ///
                /// This bit determines if the page entry is 'active' or 'inactive The
                /// CPU on x86 systems will always generate a #PF (Page Fault Exception)
                /// when this bit is not set regardless of the rest of the page table's
                /// state.
                ///
            }
            impl_bit! {read_write, get_read_write, 1
                /// Set the 'read/write' page table bit
                ///
                /// This bit determines if the page entry is able to be written to. In
                /// its default state, the page is `RO` (Read Only). When setting this
                /// bit to `true` it will allow access to `RW` (Read/Write) to the memory
                /// pointed to by this page entry.
            }
            impl_bit! {user_access, get_user_access, 2
                /// Set the 'superuser/user' page table bit.
                ///
                /// This bit determines if the page table is accessable from userspace.
                /// In its default state, page entries are **not** accessable from userspace,
                /// and will cause a #PF (Page Fault Exception).
            }
            impl_bit! {write_though, get_write_though, 3
                /// Set the 'write though' page table bit.
                ///
                /// This bit determains if this page table entry is able to do write though
                /// caching.
            }
            impl_bit! {cache_disable, get_cache_disable, 4
                /// Set the 'cach disable' page table bit.
                ///
                /// This bit determains if this page table entry is able to be cahced or not,
                /// when disabled this page table is never cached.
            }
            impl_bit! {accessed, get_accessed, 5
                /// Set the 'accessed' page table bit.
                ///
                /// This bit determains if the CPU has read from the memory to which this page
                /// points or not.
            }
            impl_bit! {page_size, get_page_size, 6
                /// Set the 'page size' page table bit.
                ///
                /// This bit determains if the page table entry is the lowest entry in the
                /// mapping, and does not point to further down page tables. Often used to
                /// map large sections of memory instead of 4kib sections.
                ///
                /// # Note
                /// This `page size` bit is only allowed on supported systems, and should
                /// only be set on compoatable page table entries. If this page table entry
                /// is not supported, the CPU will generate a #PF (Page Fault Excpetion) of
                /// a 'reserved' value.
            }
            impl_bit! {execute_disable, get_execute_disable, 7
                /// Set the 'execute disable' page table bit.
                ///
                /// This bit determains if CPU execution is allowed in this area.
            }
        })*
    };
}

impl_entry!(
    /// A Page Table Entry
    ///
    PageEntryLvl1,
    /// A Page Directry Entry field.
    ///
    PageEntryLvl2,
    /// A Page Directry Pointer Entry field.
    ///
    PageEntryLvl3,
    /// A Page Map Level 4 Entry.
    ///
    PageEntryLvl4,
    /// A Page Map Level 5 Entry.
    ///
    PageEntryLvl5
);
