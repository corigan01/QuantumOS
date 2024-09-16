/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
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

use crate::tiny_panic::fail;

#[repr(C)]
pub struct DiskAccessPacket {
    packet_size: u8,
    always_zero: u8,
    pub sectors: u16,
    pub base_ptr: u16,
    pub base_segment: u16,
    pub lba: u64,
}

impl DiskAccessPacket {
    pub fn new(sectors: u16, lba: u64, ptr: u32) -> Self {
        let base_segment = (ptr >> 4) as u16;
        let base_ptr = ptr as u16 & 0xF;

        Self {
            packet_size: 0x10,
            always_zero: 0,
            sectors,
            base_ptr,
            base_segment,
            lba,
        }
    }

    #[inline(never)]
    pub fn read(&self, disk: u16) {
        let packet_address = self as *const Self as u16;
        let status: u16;

        unsafe {
            core::arch::asm!("
                push si
                mov si, {packet:x}
                mov ax, 0x4200
                int 0x13
                jc 3f
                mov {status:x}, 0
                jmp 4f
                3:
                mov {status:x}, 1
                4:
                pop si
            ",
                in("dx") disk,
                packet = in(reg) packet_address,
                status = out(reg) status,
            );
        };

        // If the interrupt failed, we want to abort and tell the user
        if status == 1 {
            fail(b'D');
        }
    }
}
