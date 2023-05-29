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

use crate::gfx::{FramebufferPixelLayout, PixelLocation};
use crate::gfx::rectangle::Rect;

#[derive(Clone, Copy, Debug)]
pub struct FrameInfo {
    pub depth: usize,
    pub size: PixelLocation,
    pub stride: usize,
    pub total_bytes: usize,
    pub pixel_layout: FramebufferPixelLayout,
}

impl FrameInfo {
    pub fn new(
        size_x: usize,
        size_y: usize,
        depth: usize,
        stride: usize,
        total_bytes: usize,
        pixel_layout: FramebufferPixelLayout,
    ) -> Self {
        FrameInfo {
            depth,
            stride,
            total_bytes,
            pixel_layout,
            size: PixelLocation::new(size_x, size_y),
        }
    }

    pub const fn is_rect_inside_view_port(&self, rect: Rect) -> bool {
        self.is_location_inside_view_port(rect.start) &&
            self.is_location_inside_view_port(rect.end)
    }

    pub const fn is_location_inside_view_port(&self, loc: PixelLocation) -> bool {
        let size_x = self.size.x;
        let size_y = self.size.y;

        let end_x = loc.x;
        let end_y = loc.y;

        size_x >= end_x && size_y >= end_y
    }

    pub fn calculate_linear_ptr_offset(&self, loc: PixelLocation) -> usize {
        let y_offset = loc.y * self.size.x;
        let x_offset = loc.x;

        y_offset + x_offset
    }
}

