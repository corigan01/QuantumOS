/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
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

use crate::bios_ints::BiosInt;

#[repr(packed, C)]
#[derive(Debug)]
pub struct BasicVesaInfo {
    signature: [u8; 4],
    version: u16,
    oem_string_ptr: [u16; 2],
    capabilities: [u8; 4],
    video_mode_ptr: [u16; 2],
    size_64k_blocks: u16
}

impl BasicVesaInfo {
    fn new_zero() -> Self {
        Self {
            signature: [0_u8; 4],
            version: 0,
            oem_string_ptr: [0_u16; 2],
            capabilities: [0_u8; 4],
            video_mode_ptr: [0_u16; 2],
            size_64k_blocks: 0,
        }
    }

    pub fn new() -> Self {
        let mut info = Self::new_zero();

        unsafe {
            BiosInt::read_vbe_info(&mut info as *mut BasicVesaInfo as *mut u8)
                .execute_interrupt();
        }


        info
    }
}