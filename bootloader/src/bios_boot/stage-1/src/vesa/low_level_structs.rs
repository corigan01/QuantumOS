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

use quantum_lib::x86_64::bios_call::BiosCall;
use bootloader::error::BootloaderError;
use quantum_lib::ptr::segmented_ptr::SegPtr;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct VesaModeInfo {
    pub attributes: u16,
    window_a: u8,
    window_b: u8,
    granularity: u16,
    window_size: u16,
    segment_a: u16,
    segment_b: u16,
    win_function_ptr: u32,
    pub pitch: u16,
    pub width: u16,
    pub height: u16,
    w_char: u8,
    y_char: u8,
    planes: u8,
    pub bpp: u8,
    banks: u8,
    memory_model: u8,
    bank_size: u8,
    image_pages: u8,
    reserved1: u8,
    red_mask: u8,
    red_pos: u8,
    green_mask: u8,
    green_pos: u8,
    blue_mask: u8,
    blue_pos: u8,
    reserved_mask: u8,
    reserved_pos: u8,
    color_attributes: u8,
    pub framebuffer: u32,
    off_screen_memory_offset: u32,
    off_screen_memory_size: u16,
    reserved2: [u8; 206],
}

impl Default for VesaModeInfo {
    fn default() -> Self {
        Self {
            attributes: 0,
            window_a: 0,
            window_b: 0,
            granularity: 0,
            window_size: 0,
            segment_a: 0,
            segment_b: 0,
            win_function_ptr: 0,
            pitch: 0,
            width: 0,
            height: 0,
            w_char: 0,
            y_char: 0,
            planes: 0,
            bpp: 0,
            banks: 0,
            memory_model: 0,
            bank_size: 0,
            image_pages: 0,
            reserved1: 0,
            red_mask: 0,
            red_pos: 0,
            green_mask: 0,
            green_pos: 0,
            blue_mask: 0,
            blue_pos: 0,
            reserved_mask: 0,
            reserved_pos: 0,
            color_attributes: 0,
            framebuffer: 0,
            off_screen_memory_offset: 0,
            off_screen_memory_size: 0,
            reserved2: [0; 206],
        }
    }
}

impl VesaModeInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn quarry(mode_id: usize) -> Result<Self, BootloaderError> {
        let mut this_mode = Self::new();
        let ptr = &mut this_mode as *mut VesaModeInfo as *mut u8;

        assert!(mode_id <= u16::MAX as usize);
        assert!(ptr as usize <= u16::MAX as usize);

        let interrupt_status = unsafe {
            BiosCall::new()
                .bit16_call()
                .read_vbe_mode(ptr, mode_id as u16)
        };

        if interrupt_status.did_fail() {
            return Err(BootloaderError::BiosCallFailed);
        }

        Ok(this_mode)
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C, packed(2))]
pub struct VesaDriverInfo {
    pub signature: [u8; 4],
    pub version: u16,
    pub oem_string_ptr: [u16; 2],
    pub capabilities: [u8; 4],
    pub video_mode_ptr: [u16; 2],
    pub size_64k_blocks: u16,
}

impl VesaDriverInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn quarry() -> Result<Self, BootloaderError> {
        let mut info = Self::new();
        let ptr = (&mut info) as *mut Self as *mut u8;

        assert!(ptr as u32 <= u16::MAX as u32);

        let status = unsafe { BiosCall::new().bit16_call().read_vbe_info(ptr) };

        if status.did_fail() || !info.validate_signature() {
            return Err(BootloaderError::BiosCallFailed);
        }

        Ok(info)
    }

    pub fn validate_signature(&self) -> bool {
        self.signature == *b"VESA"
    }

    pub fn get_mode_ptr(&self) -> *mut u16 {
        assert!(self.validate_signature());

        SegPtr::new(self.video_mode_ptr[1], self.video_mode_ptr[0])
            .unsegmentize() as *mut u16
    }

    pub fn get_supported_modes_len(&self) -> usize {
        let ptr = self.get_mode_ptr();
        let mut offset = 0;

        loop {
            let data = unsafe { *ptr.add(offset) };

            if data == 0 || offset >= 256 || data > 6000 {
                return offset - 1;
            }

            offset += 1;
        }
    }

    pub fn get_all_supported_modes(&self) -> &[u16] {
        unsafe {
            core::slice::from_raw_parts(self.get_mode_ptr(), self.get_supported_modes_len())
        }
    }
}
