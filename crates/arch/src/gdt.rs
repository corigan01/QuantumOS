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

use core::arch::asm;
use hw::make_hw;

#[repr(C)]
pub struct GlobalDescriptorTable<const TABLE_SIZE: usize>([u64; TABLE_SIZE]);

impl<const TABLE_SIZE: usize> GlobalDescriptorTable<TABLE_SIZE> {
    pub const fn new() -> Self {
        Self([0; TABLE_SIZE])
    }

    pub fn store(&mut self, loc: usize, entry: impl SegmentEntry) {
        assert!(
            loc > 0,
            "Cannot set zero entry! Bottom entiry must be always zero!"
        );
        self.0[loc] = entry.into_entry();
    }

    pub fn pack(&'static self) -> GdtPointer {
        GdtPointer {
            limit: (TABLE_SIZE * size_of::<u64>() - 1) as u16,
            base: self.0.as_ptr(),
        }
    }
}

#[repr(C, packed(2))]
#[allow(unused)]
pub struct GdtPointer {
    limit: u16,
    base: *const u64,
}

impl GdtPointer {
    pub unsafe fn load(self) {
        asm!("lgdt [{}]", in(reg) &self);
    }
}

#[make_hw(
    field(RW, 0..16, segment_limit_lo),
    field(RW, 16..32, base_address_lo),
    field(RW, 32..40, base_address_mi),
    field(RW, 40, pub accessed),
    field(RW, 41, pub writable),
    field(RW, 42, pub direction),
    // default = true
    field(RW, 44, user_segment),
    field(RW, 45..47, privilege_level),
    field(RW, 47, pub present),
    field(RW, 47..52, segment_limit_hi),
    field(RW, 52, undef),
    field(RW, 53, long_mode),
    field(RW, 54, big),
    field(RW, 55, granularity),
    field(RW, 56..64, base_address_hi)
)]
#[derive(Clone, Copy)]
pub struct DataSegmentDesc(u64);

#[make_hw(
    field(RW, 0..16, segment_limit_lo),
    field(RW, 16..32, base_address_lo),
    field(RW, 32..40, base_address_mi),
    field(RW, 40, pub accessed),
    field(RW, 41, pub writable),
    field(RW, 42, pub conforming),
    // default = true
    // this is also the `executable` flag
    field(RW, 43, code_segment),
    // default = true
    field(RW, 44, user_segment),
    field(RW, 45..47, pub privilege_level),
    field(RW, 47, pub present),
    field(RW, 47..52, segment_limit_hi),
    field(RW, 52, undef),
    field(RW, 53, long_mode),
    field(RW, 54, big),
    field(RW, 55, granularity),
    field(RW, 56..64, base_address_hi)
)]
#[derive(Clone, Copy)]
pub struct CodeSegmentDesc(u64);

impl DataSegmentDesc {
    pub const fn new64() -> Self {
        Self(0).set_user_segment_flag(true)
    }
}

impl CodeSegmentDesc {
    pub const fn new64() -> Self {
        Self(0)
            .set_user_segment_flag(true)
            .set_code_segment_flag(true)
            .set_long_mode_flag(true)
    }
}

pub trait SegmentEntry {
    fn into_entry(self) -> u64;
}

impl SegmentEntry for CodeSegmentDesc {
    fn into_entry(self) -> u64 {
        self.0
    }
}

impl SegmentEntry for DataSegmentDesc {
    fn into_entry(self) -> u64 {
        self.0
    }
}
