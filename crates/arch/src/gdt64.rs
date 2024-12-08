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

use hw::make_hw;

pub struct GlobalDescriptorTable<const TABLE_SIZE: usize>([u64; TABLE_SIZE]);

impl<const TABLE_SIZE: usize> GlobalDescriptorTable<TABLE_SIZE> {
    pub const fn new() -> Self {
        Self([0; TABLE_SIZE])
    }
}

#[make_hw(
    field(RW, 0..=15, limit_lo),
    field(RW, 16..=39, base_lo),

    field(RW, 40, accessed),
    field(RW, 41, writable),
    /// Direction bit. If clear (0) the segment grows up. If set (1) the segment grows down, 
    /// ie. the Offset has to be greater than the Limit.
    ///
    /// ### For data selectors only!
    ///
    /// - (https://osdev.wiki/wiki/Global_Descriptor_Table)
    field(RW, 42, direction),
    /// Conforming bit.
    ///
    /// If clear (0) code in this segment can only be executed from the ring set in DPL.
    /// If set (1) code in this segment can be executed from an equal or lower privilege level. For example, 
    /// code in ring 3 can far-jump to conforming code in a ring 2 segment. The DPL field represent the 
    /// highest privilege level that is allowed to execute the segment. For example, code in ring 0 cannot 
    /// far-jump to a conforming code segment where DPL is 2, while code in ring 2 and 3 can. Note that 
    /// the privilege level remains the same, ie. a far-jump from ring 3 to a segment with a DPL of 2 remains 
    /// in ring 3 after the jump.
    ///
    /// ### For code selectors only!
    ///
    /// - (https://osdev.wiki/wiki/Global_Descriptor_Table)
    field(RW, 42, conforming),
    field(RW, 43, executable),
    field(RW, 44, descriptor_type),
    field(RW, 45..=46, dpl),
    field(RW, 47, present),

    field(RW, 48..=51, limit_hi),
    field(RW, 52..=55, flags),
    field(RW, 56..=63, base_hi)
)]
pub struct SegmentDescriptor(u64);

impl SegmentDescriptor {
    pub fn new() -> Self {
        Self(0)
    }
}
