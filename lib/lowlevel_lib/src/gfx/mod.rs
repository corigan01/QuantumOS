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

use core::ops::{Add, AddAssign, Sub, SubAssign};

pub mod draw_packet;
pub mod frame_info;
pub mod rectangle;
pub mod linear_framebuffer;
pub mod bitmap_font;

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

impl Add for PixelLocation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let x = self.x + rhs.x;
        let y = self.y + rhs.y;

        Self::new(x, y)
    }
}

impl AddAssign for PixelLocation {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
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
    pub const RED: Pixel =   Pixel::from_hex(0xFF0000);
    pub const GREEN: Pixel = Pixel::from_hex(0x00FF00);
    pub const BLUE: Pixel =  Pixel::from_hex(0x0000FF);
    pub const WHITE: Pixel = Pixel::from_hex(0xFFFFFF);
    pub const BLACK: Pixel = Pixel::from_hex(0x000000);

    pub const fn from_hex(hex: u32) -> Self {
        Pixel {
            red: ((hex & 0xFF0000) >> 16) as u8,
            green: ((hex & 0x00FF00) >> 8) as u8,
            blue: (hex & 0x0000FF) as u8,
            alpha: 255
        }
    }

    pub const fn to_hex_with_layout(&self, layout: FramebufferPixelLayout) -> u32 {
        match layout {
            FramebufferPixelLayout::RGB => {
                ((self.red as u32) << 2*8)          |
                    ((self.green as u32) << 1*8)    |
                    ((self.blue as u32) << 0*8)
            }
            FramebufferPixelLayout::GRB => {
                ((self.red as u32) << 1*8)          |
                    ((self.green as u32) << 2*8)    |
                    ((self.blue as u32) << 0*8)
            }
            FramebufferPixelLayout::BGR => {
                ((self.red as u32) << 0*8)          |
                    ((self.green as u32) << 1*8)    |
                    ((self.blue as u32) << 2*8)
            }
            FramebufferPixelLayout::WB => {
                unimplemented!()
            }
            FramebufferPixelLayout::Unknown => {
                unimplemented!()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::gfx::{FramebufferPixelLayout, Pixel};

    #[test]
    pub fn test_red_pixel_value() {
        let pixel = Pixel::RED;

        assert_eq!(pixel.red, 0xFF);
        assert_eq!(pixel.green, 0x00);
        assert_eq!(pixel.blue, 0x00);
        assert_eq!(pixel.alpha, 0xFF);

        let pixel = pixel.to_hex_with_layout(FramebufferPixelLayout::RGB);

        assert_eq!(pixel, 0xFF0000);

    }

    #[test]
    pub fn test_green_pixel_value() {
        let pixel = Pixel::GREEN;

        assert_eq!(pixel.red, 0x00);
        assert_eq!(pixel.green, 0xFF);
        assert_eq!(pixel.blue, 0x00);
        assert_eq!(pixel.alpha, 0xFF);

        let pixel = pixel.to_hex_with_layout(FramebufferPixelLayout::RGB);

        assert_eq!(pixel, 0x00FF00);
    }

    #[test]
    pub fn test_blue_pixel_value() {
        let pixel = Pixel::BLUE;

        assert_eq!(pixel.red, 0x00);
        assert_eq!(pixel.green, 0x00);
        assert_eq!(pixel.blue, 0xFF);
        assert_eq!(pixel.alpha, 0xFF);

        let pixel = pixel.to_hex_with_layout(FramebufferPixelLayout::RGB);

        assert_eq!(pixel, 0x0000FF);
    }

    #[test]
    pub fn test_white_pixel_value() {
        let pixel = Pixel::WHITE;

        assert_eq!(pixel.red, 0xFF);
        assert_eq!(pixel.green, 0xFF);
        assert_eq!(pixel.blue, 0xFF);
        assert_eq!(pixel.alpha, 0xFF);

        let pixel = pixel.to_hex_with_layout(FramebufferPixelLayout::RGB);

        assert_eq!(pixel, 0xFFFFFF);
    }

    #[test]
    pub fn test_black_pixel_value() {
        let pixel = Pixel::BLACK;

        assert_eq!(pixel.red, 0x00);
        assert_eq!(pixel.green, 0x00);
        assert_eq!(pixel.blue, 0x00);
        assert_eq!(pixel.alpha, 0xFF);

        let pixel = pixel.to_hex_with_layout(FramebufferPixelLayout::RGB);

        assert_eq!(pixel, 0x000000);
    }
}