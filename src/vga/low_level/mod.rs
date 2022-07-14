/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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

use core::{ops, slice};
use bootloader::boot_info::{FrameBuffer, Optional, PixelFormat};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FBuffer {
    buffer: Option<u64>,
    stride: usize,
    bytes_per_pixel: usize,
    resl: (usize, usize),
    pixel_format: PixelFormat
}

fn u32_to_rgb(color: u32) -> (u32, u32, u32) {
    (color & 0xFF0000 / 0x10000, (color & 0x00FF00) / 0x100, (color & 0x0000FF))
}

impl FBuffer {
    pub fn new(fb: &Optional<FrameBuffer>) -> FBuffer {
        if let Some(framebuffer) = fb.as_ref() {
            let framebuffer_info = framebuffer.info();

             FBuffer {
                buffer: Option::Some(framebuffer.buffer().as_ptr() as u64),
                stride: framebuffer_info.stride,
                bytes_per_pixel: framebuffer_info.bytes_per_pixel,
                resl: (framebuffer_info.horizontal_resolution, framebuffer_info.vertical_resolution),
                pixel_format: framebuffer_info.pixel_format
            }
        }
        else {
            panic!("Framebuffer does not contain a valid buffer!");
        }
    }

    pub fn draw_pixel(&self, pos: (usize, usize), color: u32) {
        // check if the pointer exists
        if let Some(data) = self.buffer {
            // Check if the pixel is inside the buffer before we unsafe draw
            if self.resl.0 <= pos.0 || self.resl.1 <= pos.1 {
                panic!("Tried to draw a pixel at ({}, {}), but screen size is only ({}, {})!",
                       pos.0, pos.1, self.resl.0, self.resl.1);
            }

            // Do the buffer calculations
            let y_offset = self.stride * pos.1 * self.bytes_per_pixel;
            let buffer_index = pos.0 * self.bytes_per_pixel + y_offset;
            let total_buffer_size =
                (self.resl.0 * self.resl.1 + (self.resl.1 * self.stride + 1)) * self.bytes_per_pixel;

            // Translate the color from 'color' into pixel_format's "color"
            // e.g pixel_format tells us that the colors are in BGR instead of RGB
            let real_color = match self.pixel_format {
                PixelFormat::U8 => {
                    let (r, g, b) = u32_to_rgb(color);

                    (r + g + b) / 3
                },
                _ => color,
            };

            // This is the drawing of the pixel into the buffer!
            unsafe {
                let rebuffer =
                    slice::from_raw_parts_mut(data as *mut u8, total_buffer_size);

                match self.pixel_format {
                    PixelFormat::U8 => {
                        rebuffer[buffer_index] = real_color as u8;
                    },

                    _ => {
                        for i in 0..self.bytes_per_pixel {
                            rebuffer[buffer_index + i] =
                                ((real_color & (0xFF * ((0x100 as u32).pow(i as u32)))) / ((0x100 as u32).pow(i as u32))) as u8;

                        }
                    }
                }

            }
        }
        else {
            panic!("Tried to write to pointer that doesn't exist!");
        }
    }

    pub fn draw_rec(&self, pos: (usize, usize), size: (usize, usize), color: u32) {
        for y in 0..size.1 {
            for x in 0..size.0 {
                self.draw_pixel((x + pos.0, y + pos.1), color);
            }
        }
    }

}