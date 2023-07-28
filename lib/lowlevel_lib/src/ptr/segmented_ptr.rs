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

pub struct SegPtr {
    segment: u16,
    address: u16,
}

impl SegPtr {
    pub fn new(seg: u16, ptr: u16) -> Self {
        Self {
            segment: seg,
            address: ptr,
        }
    }

    pub fn segmentize(ptr: u32) -> Self {
        Self {
            segment: (ptr / 0x10) as u16,
            address: (ptr % 0x10) as u16,
        }
    }

    pub fn unsegmentize(self) -> u32 {
        ((self.segment as u32) * 0x10) + (self.address as u32)
    }
}

#[cfg(test)]
mod test {
    use crate::ptr::segmented_ptr::SegPtr;

    #[test]
    fn test_new() {
        let segment = SegPtr::new(1, 0);

        assert_eq!(segment.address, 0);
        assert_eq!(segment.segment, 1);

        let segment = SegPtr::new(2, 10);

        assert_eq!(segment.address, 10);
        assert_eq!(segment.segment, 2);
    }

    #[test]
    fn test_segmentize() {
        let unsegmented_ptr = 0x100;

        let seg_ptr = SegPtr::segmentize(unsegmented_ptr);

        assert_eq!(seg_ptr.address, 0);
        assert_eq!(seg_ptr.segment, 16);
    }

    #[test]
    fn test_unsegmentize() {
        let seg_ptr = SegPtr::new(0x10, 0);

        assert_eq!(seg_ptr.address, 0);
        assert_eq!(seg_ptr.segment, 0x10);

        let unsegmented_ptr = seg_ptr.unsegmentize();

        assert_eq!(unsegmented_ptr, 0x100);
    }
}
