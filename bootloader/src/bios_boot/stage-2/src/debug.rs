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

use core::mem;
use lazy_static::lazy_static;
use quantum_lib::basic_font::BUILT_IN_FONT;
use spin::mutex::Mutex;

#[derive(Clone, Copy)]
pub struct ConsoleFramebuffer {
    pub framebuffer: u32,
    pub bbp: usize,
    pub screen_x_res: usize,
    pub screen_y_res: usize,
}

#[derive(Clone, Copy)]
pub struct Console {
    pub x: usize,
    pub y: usize,
    pub framebuffer: Option<ConsoleFramebuffer>,
}

impl Console {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            framebuffer: None,
        }
    }
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console::new());
}

pub fn setup_framebuffer(framebuffer: u32, x_res: usize, y_res: usize, bbp: usize) {
    let framebuffer = ConsoleFramebuffer {
        framebuffer,
        bbp,
        screen_x_res: x_res,
        screen_y_res: y_res,
    };

    CONSOLE.lock().framebuffer = Some(framebuffer);
}

unsafe fn scroll_framebuffer_y(ptr: *mut u8, width: usize, height: usize, scroll_height: usize) {
    let framebuffer = core::slice::from_raw_parts_mut(ptr as *mut u32, height * width);

    for _ in 0..scroll_height {
        for e in 1..height {
            for i in 0..width {
                framebuffer[i + width * (e - 1)] = framebuffer[i + width * e];
            }
        }

        for i in 0..width {
            framebuffer[i + width * (height - 1)] = 0;
        }
    }
}

pub fn display_string(string: &str) {
    let mut console_info = CONSOLE.lock();
    let frame_info = if let Some(framebuffer_info) = &console_info.framebuffer {
        framebuffer_info
    } else {
        return;
    };

    let framebuffer = frame_info.framebuffer;
    let bbp = frame_info.bbp;
    let x_res = frame_info.screen_x_res;
    let y_res = frame_info.screen_y_res;

    let char_y_addition = 5;
    let char_x_addition = 3;

    let font = &BUILT_IN_FONT;
    let first_font_char_offset = 32;

    let bytes_per_pixel = bbp / 8;

    // FIXME: Make a better color system
    let dummy_color = [0x00, 0xFF, 0x00, 0x00];

    for character in string.bytes() {
        match character.to_ascii_uppercase() {
            b'\n' => unsafe {
                console_info.y += 1;
                console_info.x = 0;

                let glyph_height = 14;

                let y_allowed_chars = y_res / (glyph_height + char_y_addition);
                if console_info.y > y_allowed_chars {
                    scroll_framebuffer_y(
                        framebuffer as *mut u8,
                        x_res,
                        y_res,
                        glyph_height + char_y_addition,
                    );
                    console_info.y -= 1;
                }
            },

            i if i >= first_font_char_offset && i < (font.len() + 32) as u8 => {
                let character_number = character - first_font_char_offset;

                let char_glyph = &font[character_number as usize];

                let glyph_height = char_glyph.len();
                let glyph_width = 8;

                for (height, width_pixels) in char_glyph.iter().rev().enumerate() {
                    for width in 0..glyph_width {
                        let pixel_iter = 1 << (glyph_width - 1 - width);
                        let is_pixel_on = width_pixels & pixel_iter > 0;

                        if !is_pixel_on {
                            continue;
                        }

                        let y = height
                            + glyph_height * console_info.y
                            + char_y_addition * console_info.y;
                        let x =
                            width + glyph_width * console_info.x + char_x_addition * console_info.x;

                        let array_offset = x + (y * x_res);
                        let ptr = framebuffer as *mut u8;

                        for pixel in 0..bytes_per_pixel {
                            unsafe {
                                *ptr.add(array_offset * bytes_per_pixel + pixel) =
                                    dummy_color[pixel];
                            }
                        }
                    }
                }

                let allowed_chars_per_line = x_res / (glyph_width + char_x_addition);

                if console_info.x >= (allowed_chars_per_line - 1) {
                    display_string("\n");
                } else {
                    console_info.x += 1;
                }
            }

            _ => {}
        }
    }
}
