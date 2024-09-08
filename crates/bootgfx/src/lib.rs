#![no_std]

use core::ptr::write_volatile;

pub mod terminal;

/// # Color
/// A color in the binary format (u32 - r: u8, g: u8, b: u8, alpha: u8).
#[derive(Clone, Copy)]
pub struct Color(pub u32);

impl Color {
    pub const WHITE: Self = Self(0xFFFFFFFF);
    pub const QUANTUM_BACKGROUND: Self = Self(0xFF121212);
}

/// # Framebuffer
/// A struct to draw graphics into framebuffer.
pub struct Framebuffer {
    buffer: *mut Color,
    height: usize,
    width: usize,
}

impl Framebuffer {
    pub const ALLOWED_BPP: usize = 32;

    /// # New Linear
    /// Make a new framebuffer based off a linear framebuffer with zero blanking or padding.
    pub unsafe fn new_linear(
        buffer: *mut u32,
        bits_per_pixel: u8,
        height: usize,
        width: usize,
    ) -> Self {
        assert_eq!(
            bits_per_pixel,
            Self::ALLOWED_BPP as u8,
            "Only 32-bits per pixel is supported!"
        );

        Framebuffer {
            buffer: buffer.cast(),
            height,
            width,
        }
    }

    /// # Draw Pixel
    /// Draw a pixel of a color onto the framebuffer.
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x > self.width || y > self.height {
            return;
        }

        let verticality_to_linearity = y * self.width;
        unsafe {
            write_volatile(self.buffer.add(verticality_to_linearity + x), color);
        };
    }

    /// # Draw Rectangle
    /// Draw a rectangle of a color onto the framebuffer.
    pub fn draw_rec(&mut self, x: usize, y: usize, length: usize, height: usize, color: Color) {
        // TODO: Use memory functions to speed this up. However, this may never
        //       be used so I don't want to optimize it until it gets used out-
        //       side the bootloader.

        for y in y..(y + height) {
            for x in x..(x + length) {
                self.draw_pixel(x, y, color);
            }
        }
    }

    /// # Height
    /// Get the height of the framebuffer.
    pub const fn height(&self) -> usize {
        self.height
    }

    /// # Width
    /// Get the width of the framebuffer.
    pub const fn width(&self) -> usize {
        self.width
    }
}
