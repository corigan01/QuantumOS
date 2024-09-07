#![no_std]

pub mod terminal;

/// # Framebuffer
/// A struct to draw graphics into framebuffer.
pub struct Framebuffer {
    buffer: *mut u32,
    height: u64,
    width: u64,
}

impl Framebuffer {
    /// # New Linear
    /// Make a new framebuffer based off a linear framebuffer with zero blanking or padding.
    pub unsafe fn new_linear(
        buffer: *mut u32,
        bits_per_pixel: u8,
        height: u64,
        width: u64,
    ) -> Self {
        assert_eq!(bits_per_pixel, 32, "Only 32-bits per pixel is supported!");

        Framebuffer {
            buffer,
            height,
            width,
        }
    }
}
