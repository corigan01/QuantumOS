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

use core::ptr::NonNull;
use quantum_lib::gfx::linear_framebuffer::LinearFramebuffer;
use qk_alloc::circular_buffer::CircularBuffer;
use quantum_lib::basic_font::BuiltInFont;
use quantum_lib::gfx::Pixel;

pub struct TextInfo {
    cursor_x: usize,
    cursor_y: usize,
    screen_x: usize,
    screen_y: usize
}

impl TextInfo {
    pub fn new(screen_x: usize, screen_y: usize) -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            screen_x,
            screen_y
        }
    }
}

pub struct KernelConsole {
    buf: CircularBuffer<u8>,
    framebuffer: NonNull<LinearFramebuffer>,
    text_info: TextInfo
}

impl KernelConsole {
    // FIXME: We need a better way to drawing to the framebuffer then
    //        taking it by ref!
    pub fn new(framebuffer: &mut LinearFramebuffer) -> Self {
        let framebuffer_info = framebuffer.info;
        let screen_x = framebuffer_info.size.x / BuiltInFont::WIDTH;
        let screen_y = framebuffer_info.size.y / BuiltInFont::HEIGHT;

        Self {
            buf: CircularBuffer::new(screen_x * screen_y),
            framebuffer: NonNull::from(framebuffer),
            text_info: TextInfo::new(screen_x, screen_y)
        }
    }

    pub fn draw(&mut self) {
        let framebuffer = unsafe { self.framebuffer.as_mut() };
        framebuffer.fill_entire(Pixel::from_hex(0x111111)).unwrap();



    }
}