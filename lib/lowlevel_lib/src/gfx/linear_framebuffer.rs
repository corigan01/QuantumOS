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

use core::ops::{ControlFlow, FromResidual, Try};
use crate::gfx::frame_info::FrameInfo;
use crate::gfx::{Pixel, PixelLocation};
use crate::gfx::draw_packet::DrawPacket;
use crate::gfx::rectangle::Rect;

#[derive(Clone, Copy, Debug)]
pub struct LinearFramebuffer {
    ptr: *mut u8,
    info: FrameInfo,
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawStatus {
    Successful,
    Failed
}

impl DrawStatus {
    pub fn unwrap(self) {
        if self == Self::Failed {
            panic!("DrawStatus Unwrap failed!");
        }
    }
}

impl FromResidual for DrawStatus {
    fn from_residual(_residual: <Self as Try>::Residual) -> Self {
        todo!()
    }
}

impl Try for DrawStatus {
    type Output = ();
    type Residual = ();

    fn from_output(_output: Self::Output) -> Self {
        todo!()
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Successful => ControlFlow::Continue(()),
            Self::Failed => ControlFlow::Break(())
        }
    }
}

impl LinearFramebuffer {
    pub fn new(ptr: *mut u8, info: FrameInfo) -> Self {
        Self {
            ptr,
            info
        }
    }

    pub const fn ptr(&self) -> *mut u8 {
        self.ptr
    }

    #[inline]
    pub fn draw_pixel(&mut self, color: Pixel, location: PixelLocation) -> DrawStatus {
        if !self.info.is_location_inside_view_port(location) {
            return DrawStatus::Failed;
        }

        let ptr_base_offset = self.info.calculate_linear_ptr_offset(location);

        // FIXME: We should re-calculate the color to fit the viewport color depth
        assert_eq!(self.info.depth, 32,
                   "FIXME: Currently we do not support color depths lower then 32-bits / pixel");

        let pixel_value = color.to_hex_with_layout(self.info.pixel_layout) << 8;

        unsafe {
            let modified_ptr = (self.ptr as *mut u32).add(ptr_base_offset);
            (*modified_ptr) = pixel_value.swap_bytes();
        }

        DrawStatus::Successful
    }

    #[inline]
    pub fn draw_packet(&mut self, packet: DrawPacket) -> DrawStatus {
        if !self.info.is_rect_inside_view_port(packet.rect) {
            return DrawStatus::Failed;
        }

        let size_x = packet.rect.size_x();

        for (index, pixel) in packet.raw_data.iter().enumerate() {
            let packet_pos_x = packet.rect.start.x + (index % size_x);
            let packet_pos_y = packet.rect.start.y + (index / size_x);

            if packet_pos_x >= packet.rect.end.x || packet_pos_y >= packet.rect.end.y {
                break;
            }

            let pixel_location = PixelLocation::new(packet_pos_x, packet_pos_y);

            self.draw_pixel(*pixel, pixel_location)?;
        }

        DrawStatus::Successful
    }

    #[inline]
    pub fn draw_rect(&mut self, rect: Rect, fill: Pixel) -> DrawStatus {
        if !self.info.is_rect_inside_view_port(rect) {
            return DrawStatus::Failed;
        }

        for y in rect.start.y..rect.end.y {
            for x in rect.start.x..rect.end.x {
                self.draw_pixel(fill, PixelLocation::new(x, y))?;
            }
        }

        DrawStatus::Successful
    }

    //pub fn draw_built_in_glyph(&mut self, c: char, location: PixelLocation) -> DrawStatus {
    //    todo!()
    //}
}


#[cfg(test)]
mod test {
    extern crate std;
    extern crate alloc;

    use alloc::vec;
    use alloc::vec::Vec;
    use core::mem::MaybeUninit;
    use crate::gfx::frame_info::FrameInfo;
    use crate::gfx::linear_framebuffer::LinearFramebuffer;
    use crate::gfx::{FramebufferPixelLayout, Pixel, PixelLocation};

    const FRAMEBUFFER_X: usize = 1920;
    const FRAMEBUFFER_Y: usize = 1080;
    const FRAMEBUFFER_DEPTH: usize = 32;
    const FRAMEBUFFER_BYTES_PER_LINE: usize = FRAMEBUFFER_X * (FRAMEBUFFER_DEPTH / 8);
    const FRAMEBUFFER_SIZE_BYTES: usize = FRAMEBUFFER_Y * FRAMEBUFFER_BYTES_PER_LINE;

    // We need to over align this buffer so we dont try to draw a pixel unaligned
    #[repr(align(4))]
    struct FakeFramebuffer(Vec<u8>);

    impl FakeFramebuffer {
        pub fn new() -> Self {
            let mut vector = vec![0; FRAMEBUFFER_SIZE_BYTES];

            vector[FRAMEBUFFER_SIZE_BYTES - 1] = 1;

            Self(vector)
        }

        pub fn as_ptr(&self) -> *const u8 {
            self.0.as_ptr()
        }

        pub fn as_mut_ptr(&mut self) -> *mut u8 {
            self.0.as_mut_ptr()
        }

        pub fn get(&self, index: usize) -> u8 {
            self.0[index]
        }
        pub fn get_32(&self, index: usize) -> u32 {
            unsafe { *((self.as_ptr() as *const u32).add(index)) }
        }
    }

    static mut TEST_FRAMEBUFFER_ADDRESS_SPACE: MaybeUninit<FakeFramebuffer> = MaybeUninit::uninit();

    fn setup_test_linear_framebuffer() -> LinearFramebuffer {
        let info = FrameInfo {
            depth: 32,
            size: PixelLocation::new(FRAMEBUFFER_X, FRAMEBUFFER_Y),
            stride: FRAMEBUFFER_BYTES_PER_LINE,
            total_bytes: FRAMEBUFFER_SIZE_BYTES,
            pixel_layout: FramebufferPixelLayout::RGB,
        };

        unsafe { TEST_FRAMEBUFFER_ADDRESS_SPACE = MaybeUninit::new(FakeFramebuffer::new()) };
        let ptr = unsafe { TEST_FRAMEBUFFER_ADDRESS_SPACE.assume_init_mut().as_mut_ptr() };

        LinearFramebuffer {
            ptr,
            info,
        }
    }

    #[test]
    fn test_drawing_red_pixel() {
        let mut framebuffer = setup_test_linear_framebuffer();

        framebuffer.draw_pixel(Pixel::RED, PixelLocation::new(0, 0));

        let test_address_space = unsafe {
            TEST_FRAMEBUFFER_ADDRESS_SPACE.assume_init_mut()
        };

        assert_eq!(test_address_space.get(0), 0xFF_u8);
        assert_eq!(test_address_space.get(1), 0x00_u8);
        assert_eq!(test_address_space.get(2), 0x00_u8);
        assert_eq!(test_address_space.get(3), 0x00_u8);
    }

    #[test]
    fn test_drawing_green_pixel() {
        let mut framebuffer = setup_test_linear_framebuffer();

        framebuffer.draw_pixel(Pixel::GREEN, PixelLocation::new(0, 0));

        let test_address_space = unsafe {
            TEST_FRAMEBUFFER_ADDRESS_SPACE.assume_init_mut()
        };

        assert_eq!(test_address_space.get(0), 0x00_u8);
        assert_eq!(test_address_space.get(1), 0xFF_u8);
        assert_eq!(test_address_space.get(2), 0x00_u8);
        assert_eq!(test_address_space.get(3), 0x00_u8);
    }

    #[test]
    fn test_drawing_blue_pixel() {
        let mut framebuffer = setup_test_linear_framebuffer();

        framebuffer.draw_pixel(Pixel::BLUE, PixelLocation::new(0, 0));

        let test_address_space = unsafe {
            TEST_FRAMEBUFFER_ADDRESS_SPACE.assume_init_mut()
        };

        assert_eq!(test_address_space.get(0), 0x00_u8);
        assert_eq!(test_address_space.get(1), 0x00_u8);
        assert_eq!(test_address_space.get(2), 0xFF_u8);
        assert_eq!(test_address_space.get(3), 0x00_u8);
    }

    #[test]
    fn test_drawing_white_pixel() {
        let mut framebuffer = setup_test_linear_framebuffer();

        framebuffer.draw_pixel(Pixel::WHITE, PixelLocation::new(0, 0));

        let test_address_space = unsafe {
            TEST_FRAMEBUFFER_ADDRESS_SPACE.assume_init_mut()
        };

        assert_eq!(test_address_space.get(0), 0xFF_u8);
        assert_eq!(test_address_space.get(1), 0xFF_u8);
        assert_eq!(test_address_space.get(2), 0xFF_u8);
        assert_eq!(test_address_space.get(3), 0x00_u8);
    }

    #[test]
    fn test_drawing_black_pixel() {
        let mut framebuffer = setup_test_linear_framebuffer();

        framebuffer.draw_pixel(Pixel::BLACK, PixelLocation::new(0, 0));

        let test_address_space = unsafe {
            TEST_FRAMEBUFFER_ADDRESS_SPACE.assume_init_mut()
        };

        assert_eq!(test_address_space.get(0), 0x00_u8);
        assert_eq!(test_address_space.get(1), 0x00_u8);
        assert_eq!(test_address_space.get(2), 0x00_u8);
        assert_eq!(test_address_space.get(3), 0x00_u8);
    }

}