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

use crate::debug_println;
use crate::error_utils::QuantumError;

pub type RawColor = (u8, u8, u8, u8);
pub type FramePixelHandler = fn(usize, usize, RawColor);

pub struct FrameBuffer {
    pixel_stream: Option<FramePixelHandler>,
    x_size: usize,
    y_size: usize,
}

impl FrameBuffer {
    pub fn new() -> Self {
        Self {
            pixel_stream: None,
            x_size: 0,
            y_size: 0
        }
    }

    pub fn set_output_stream(&mut self, stream: FramePixelHandler, x: usize, y: usize) {
        self.pixel_stream = Some(stream);
        self.x_size = x;
        self.y_size = y;
    }

    pub fn draw_pixel(&self, pos: (usize, usize), color: RawColor) -> Result<(), QuantumError> {
        let pos_x = pos.0;
        let pos_y = pos.1;

        // cant draw outside the buffer size
        if pos_x > self.x_size || pos_y > self.y_size {
            return Err(QuantumError::OutOfRange);
        }

        if let Some(stream) = self.pixel_stream {
            let stream = stream as FramePixelHandler;

            stream(pos_x, pos_y, color);
        } else {
            return Err(QuantumError::NoStream)
        }

        Ok(())
    }

    pub fn draw_rec(&self, pos: (usize, usize), size: (usize, usize), color: RawColor) ->  Result<(), QuantumError> {
        for y in pos.1..size.1 {
            for x in pos.0..size.0 {
                self.draw_pixel((x, y), color)?;
            }
        }
        
        Ok(())
    }


}