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

use crate::gfx::PixelLocation;

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub start: PixelLocation,
    pub end: PixelLocation,
}

impl Rect {
    pub fn new(start: PixelLocation, end: PixelLocation) -> Self {
        Self {
            start,
            end
        }
    }

    pub fn dist(start: PixelLocation, size: PixelLocation) -> Self {
        Self {
            start,
            end: start + size
        }
    }

    pub fn pixel_area(&self) -> usize {
        let area = self.end - self.start;

        area.x * area.y
    }

    pub fn size_x(&self) -> usize {
        self.end.x.abs_diff(self.start.x)
    }

    pub fn size_y(&self) -> usize {
        self.end.y.abs_diff(self.end.y)
    }
}

#[macro_export]
macro_rules! rect {
    ($x:literal, $y:literal, $xx:literal, $yy:literal) => {
        Rect::new(PixelLocation::new($x, $y), PixelLocation::new($xx, $yy))
    };

    ($x:literal, $y:literal ; $xx:literal, $yy:literal) => {
        Rect::dist(PixelLocation::new($x, $y), PixelLocation::new($xx, $yy))
    }
}