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

#![allow(dead_code)]

use core::ops::{Sub, SubAssign};
use crate::bitset::BitSet;

pub mod draw_packet;
pub mod frame_info;
pub mod rectangle;
pub mod linear_framebuffer;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FramebufferPixelLayout {
    RGB,
    GRB,
    BGR,
    WB,

    Unknown,
}

#[derive(Clone, Copy, Debug)]
pub struct PixelLocation {
    pub x: usize,
    pub y: usize,
}

impl PixelLocation {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y
        }
    }
}

impl Sub for PixelLocation {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let x = self.x - rhs.x;
        let y = self.y - rhs.y;

        Self::new(x, y)
    }
}

impl SubAssign for PixelLocation {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Default for PixelLocation {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Pixel {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Pixel {
    pub fn stream_into_u32(&self, layout: FramebufferPixelLayout) -> u32 {
        // FIXME: We should be able to use not only RGB layouts
        assert_eq!(layout, FramebufferPixelLayout::RGB,
            "FIXME: Currently we only support RGB pixel layout");


        let mut value: u32 = 0;
        value.set_bits(24..32, self.red as u64);
        value.set_bits(16..24, self.green as u64);
        value.set_bits(8..16, self.blue as u64);
        value.set_bits(0..8, self.alpha as u64);


        value
    }
}