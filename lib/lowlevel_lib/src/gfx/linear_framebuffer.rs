/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
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

use crate::gfx::frame_info::FrameInfo;
use crate::gfx::{Pixel, PixelLocation};
use crate::gfx::draw_packet::DrawPacket;

#[derive(Clone, Copy, Debug)]
pub struct LinearFramebuffer {
    ptr: *mut u8,
    info: FrameInfo,
}

pub enum DrawStatus {
    Successful,
    Failed
}

impl LinearFramebuffer {
    pub fn new(ptr: *mut u8, info: FrameInfo) -> Self {
        Self {
            ptr,
            info
        }
    }

    pub fn draw_pixel(&mut self, color: Pixel, location: PixelLocation) -> DrawStatus {
        if !self.info.is_location_inside_view_port(location) {
            return DrawStatus::Failed;
        }

        let ptr_base_offset = self.info.calculate_linear_ptr_offset(location);

        // FIXME: We should re-calculate the color to fit the viewport color depth
        assert_eq!(self.info.depth, 32,
                   "FIXME: Currently we do not support color depths lower then 32-bits / pixel");

        let pixel_value = color.stream_into_u32(self.info.pixel_layout);

        unsafe {
            let modified_ptr = (self.ptr as *mut u32).add(ptr_base_offset);
            *modified_ptr = pixel_value;
        }

        DrawStatus::Successful
    }

    pub fn draw_packet(&mut self, packet: DrawPacket) -> DrawStatus {
        if !self.info.is_location_inside_view_port(packet.rect.end) {
            return DrawStatus::Failed;
        }

        todo!()
    }
}

