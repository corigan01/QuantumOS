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
use crate::error::BootloaderError;
use crate::{bios_println, convert_segmented_ptr};
use quantum_lib::heapless_string::HeaplessString;

#[derive(Clone, Copy, Debug)]
#[repr(packed, C)]
pub struct BasicVesaController {
    signature: [u8; 4],
    version: u16,
    oem_string_ptr: [u16; 2],
    capabilities: [u8; 4],
    video_mode_ptr: [u16; 2],
    size_64k_blocks: u16,
}

#[repr(C, packed)]
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
    banks: u8,
    memory_model: u8,
    bank_size: u8,
    image_pages: u8,
    reserved: u8,
    red_mask: u8,
    red_pos: u8,
    green_mask: u8,
    green_pos: u8,
    blue_mask: u8,
    blue_pos: u8,
    reserved_mask: u8,
    reserved_pos: u8,
    color_attributes: u8,
    framebuffer: u32,
    off_screen_memory_offset: u32,
    off_screen_memory_size: u16,
}

impl VesaModeInfo {
    pub fn new() -> Self {
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
            banks: 0,
            memory_model: 0,
            bank_size: 0,
            image_pages: 0,
            reserved: 0,
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
        }
    }
}

#[derive(Debug)]
pub struct VbeMode(usize);

impl VbeMode {
    pub fn get_mode_number(&self) -> usize {
        return 0;
    }
}

impl BasicVesaController {
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

    pub fn new() -> Result<Self, BootloaderError> {
        let mut info = Self::new_zero();
        let info_ptr = &mut info as *mut BasicVesaController as *mut u8;

        let int_status = unsafe { BiosInt::read_vbe_info(info_ptr).execute_interrupt() };

        if int_status.did_fail() {
            return Err(BootloaderError::BiosCallFailed);
        }

        if info_ptr as u16 == 0 {
            panic!("vbe null ptr");
        }

        if !info.validate_signature() {
            panic!("invalid Signature");
        }

        Ok(info)
    }

    pub fn validate_signature(&self) -> bool {
        self.signature[0] == b'V'
            && self.signature[1] == b'E'
            && self.signature[2] == b'S'
            && self.signature[3] == b'A'
    }

    pub fn get_version_number(&self) -> u16 {
        self.version
    }

    pub fn get_oem_string(&self) -> HeaplessString<64> {
        let ptr = convert_segmented_ptr((
            self.oem_string_ptr[1] as usize,
            self.oem_string_ptr[0] as usize,
        ));

        let bytes = unsafe { core::slice::from_raw_parts_mut(ptr as *mut u8, 64) };

        HeaplessString::from_bytes(bytes).unwrap()
    }

    pub fn run_on_every_supported_mode_and_return_on_true<Function>(
        &self,
        function: Function,
    ) -> Option<VbeMode>
    where
        Function: Fn(&VesaModeInfo, &VbeMode) -> bool,
    {
        let ptr = convert_segmented_ptr((
            self.video_mode_ptr[1] as usize,
            self.video_mode_ptr[0] as usize,
        )) as *mut u16;

        let mut iteration_len = 0;
        let array_size = loop {
            let data = unsafe { *ptr.add(iteration_len) };

            if data != 0 {
                iteration_len += 1;
            } else {
                break iteration_len;
            }
        };

        let modes = unsafe { core::slice::from_raw_parts_mut(ptr as *mut u16, array_size) };

        let mut temp_mode = VesaModeInfo::new();
        'outer: for mode in modes {
            for i in 0..5 {
                let bios_int_status = unsafe {
                    BiosInt::read_vbe_mode((&mut temp_mode) as *mut VesaModeInfo as *mut u8, *mode)
                        .execute_interrupt()
                };

                if bios_int_status.did_succeed() {
                    break;
                }

                if i == 4 && bios_int_status.did_fail() {
                    continue 'outer;
                }
            }

            let mode_id = VbeMode { 0: *mode as usize };

            if function(&temp_mode, &mode_id) {
                return Some(mode_id);
            }

            temp_mode = VesaModeInfo::new();
        }

        None
    }

    pub fn set_video_mode(&self, mode: VbeMode) -> Result<(), BootloaderError> {
        let modes_match =
            self.run_on_every_supported_mode_and_return_on_true(|info, checking_mode| {
                checking_mode.get_mode_number() == mode.get_mode_number()
            });

        if modes_match.is_none() {
            return Err(BootloaderError::NoValid);
        }

        let bios_int_status =
            unsafe { BiosInt::set_vbe_mode(mode.get_mode_number() as u16).execute_interrupt() };

        if bios_int_status.did_fail() {
            return Err(BootloaderError::BiosCallFailed);
        }

        bios_println!("Successfully changed video mode!");

        Ok(())
    }
}

impl Default for BasicVesaController {
    fn default() -> Self {
        Self::new_zero()
    }
}
