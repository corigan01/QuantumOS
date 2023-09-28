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

use core::cmp::min;
use core::ops::{Bound, RangeBounds};
use qk_alloc::boxed::Box;
use crate::FsResult;
use crate::io::{Read, ReadWriteSeek, Seek, SeekFrom, Write};

pub struct AbstractBuffer {
    range_start: Bound<u64>,
    range_end: Bound<u64>,
    data: Box<dyn ReadWriteSeek>,
    current_seek: u64,
}

impl AbstractBuffer {
    pub fn new<Range>(data: Box<dyn ReadWriteSeek>, readable_range: Range) -> Self
        where Range: RangeBounds<u64> {
        Self {
            range_start: readable_range.start_bound().cloned(),
            range_end: readable_range.end_bound().cloned(),
            data,
            current_seek: 0,
        }
    }

    const fn decode_bound(bound: Bound<u64>, unbounded: u64) -> u64 {
        match bound {
            Bound::Included(val) => val,
            Bound::Excluded(val) => val - 1,
            Bound::Unbounded => unbounded
        }
    }

    fn readable_range_begin(&self, buffer_end: u64) -> u64 {
        min(Self::decode_bound(self.range_start, 0), buffer_end)
    }

    fn readable_range_end(&self, buffer_end: u64) -> u64 {
        min(Self::decode_bound(self.range_end, buffer_end), buffer_end)
    }

    fn readable_range_size(&self, buffer_end: u64) -> u64 {
        self.readable_range_end(buffer_end) - self.readable_range_begin(buffer_end)
    }

    pub fn inner_range_begin(&self) -> Bound<u64> {
        self.range_start
    }

    pub fn inner_range_end(&self) -> Bound<u64> {
        self.range_end
    }

    #[inline]
    fn shrink_range<RangeOrig, RangeNew>(original: RangeOrig, new: RangeNew, buffer_end: u64) -> (Bound<u64>, Bound<u64>)
        where RangeOrig: RangeBounds<u64>,
              RangeNew: RangeBounds<u64> {

        let original_bound_begin = Self::decode_bound(
            original.start_bound().cloned(),
            0
        );

        let new_bound_begin = Self::decode_bound(
            new.start_bound().cloned(),
            0
        );

        let bound_added = original_bound_begin + new_bound_begin;
        let return_bound_begin = match bound_added {
            0 => {
                if matches!(original.start_bound(), Bound::Unbounded) && matches!(new.start_bound(), Bound::Unbounded) {
                    Bound::Unbounded
                } else {
                    Bound::Included(0)
                }
            },
            val => Bound::Included(val)
        };

        let original_bound_end = Self::decode_bound(
            original.end_bound().cloned(),
            buffer_end
        );

        let new_bound_end = Self::decode_bound(
            new.end_bound().cloned(),
            buffer_end
        );

        let return_bound_end = match new_bound_end {
            v if v == buffer_end => original.end_bound().cloned(),
            val => Bound::Included(min(val + new_bound_begin, original_bound_end))
        };

        (return_bound_begin, return_bound_end)
    }

    pub fn temporary_shrink<Func, Range, AnyOut>(&mut self, tmp_range: Range, mut func: Func) -> AnyOut
        where Func: FnMut(&mut Self) -> AnyOut,
              Range: RangeBounds<u64> {

        let current_begin = self.range_start;
        let current_end = self.range_end;
        let current_self_seek = self.current_seek;

        (self.range_start, self.range_end) = Self::shrink_range(
            (self.range_start, self.range_end),
            tmp_range,
            self.data.stream_len().unwrap()
        );

        let return_result = func(self);

        self.range_start = current_begin;
        self.range_end = current_end;
        self.current_seek = current_self_seek;

        return_result
    }

}

impl Read for AbstractBuffer {
    fn read(&mut self, buf: &mut [u8]) -> FsResult<usize> {
        let read_amount = self.data.read(buf)?;
        self.current_seek += read_amount as u64;

        Ok(read_amount)
    }
}

impl Write for AbstractBuffer {
    fn write(&mut self, buf: &mut [u8]) -> FsResult<usize> {
        let write_amount = self.data.write(buf)?;
        self.current_seek += write_amount as u64;

        Ok(write_amount)
    }

    fn flush(&mut self) -> FsResult<()> {
        self.data.flush()
    }
}

impl Seek for AbstractBuffer {
    fn seek(&mut self, pos: SeekFrom) -> FsResult<u64> {
        let buffer_size = self.data.stream_len()?;
        let abs_pos = match pos {
            SeekFrom::Start(value) => {
                self.readable_range_begin(buffer_size) + value
            }
            SeekFrom::Current(value) => {
                (self.current_seek as i64 + value) as u64
            }
            SeekFrom::End(value) => {
                (self.readable_range_size(buffer_size) as i64 + value) as u64
            }
        };

        self.data.seek(SeekFrom::Start(abs_pos))?;

        match pos {
            SeekFrom::Start(value) => {
                self.current_seek = value;
            }
            SeekFrom::Current(value) => {
                self.current_seek = ((self.current_seek as i64) + value) as u64;
            }
            SeekFrom::End(value) => {
                self.current_seek = self.readable_range_size(buffer_size);
            }
        }

        Ok(self.current_seek)
    }

    fn stream_len(&mut self) -> FsResult<u64> {
        let buffer_size = self.data.stream_len()?;
        Ok(self.readable_range_size(buffer_size))
    }
}

#[cfg(test)]
mod test {
    use core::ops::Bound;
    use qk_alloc::boxed::Box;
    use crate::abstract_buffer::AbstractBuffer;
    use crate::io::{Read, Seek, SeekFrom, Write};
    use crate::{FsResult, set_example_allocator};



    struct TestBuffer<const SIZE: usize> {
        seek_pos: u64
    }

    impl<const SIZE: usize> TestBuffer<SIZE> {
        pub fn new() -> Self {
            Self {
                seek_pos: 0
            }
        }
    }

    impl<const SIZE: usize> Seek for TestBuffer<SIZE> {
        fn seek(&mut self, pos: SeekFrom) -> FsResult<u64> {
            match pos {
                SeekFrom::Start(value) => {
                    self.seek_pos = 0;
                }
                SeekFrom::Current(value) => {
                    self.seek_pos = ((self.seek_pos as i64) + value) as u64;
                }
                SeekFrom::End(value) => {
                    self.seek_pos = ((SIZE as i64) + value) as u64;
                }
            };

            Ok(self.seek_pos)
        }
    }

    impl<const SIZE: usize> Write for TestBuffer<SIZE> {
        fn write(&mut self, buf: &mut [u8]) -> FsResult<usize> {
            Ok(buf.len())
        }

        fn flush(&mut self) -> FsResult<()> {
            Ok(())
        }
    }

    impl<const SIZE: usize> Read for TestBuffer<SIZE> {
        fn read(&mut self, buf: &mut [u8]) -> FsResult<usize> {
            Ok(buf.len())
        }
    }

    #[test]
    fn test_shrink_bounds() {
        assert_eq!(AbstractBuffer::shrink_range(0..=10, 0..=5, 10), (Bound::Included(0), Bound::Included(5)));
        assert_eq!(AbstractBuffer::shrink_range(..5, ..3, 5), (Bound::Unbounded, Bound::Included(2)));
        assert_eq!(AbstractBuffer::shrink_range(..5, ..=3, 5), (Bound::Unbounded, Bound::Included(3)));
        assert_eq!(AbstractBuffer::shrink_range(0..5, ..3, 5), (Bound::Included(0), Bound::Included(2)));
        assert_eq!(AbstractBuffer::shrink_range(0.., 2.., 5), (Bound::Included(2), Bound::Unbounded));
    }

    #[test]
    fn test_seek_no_bounds() {
        set_example_allocator(4096);
        let mut buffer = AbstractBuffer::new(
            Box::new(TestBuffer::<10>::new()),
            ..
        );

        assert_eq!(buffer.seek(SeekFrom::Start(0)), Ok(0));
        assert_eq!(buffer.seek(SeekFrom::End(0)), Ok(10));
        assert_eq!(buffer.seek(SeekFrom::Current(-1)), Ok(9));
    }

    #[test]
    fn test_seek_no_start_bound() {
        set_example_allocator(4096);
        let mut buffer = AbstractBuffer::new(
            Box::new(TestBuffer::<10>::new()),
            ..5
        );

        assert_eq!(buffer.seek(SeekFrom::Start(0)), Ok(0));
        assert_eq!(buffer.seek(SeekFrom::End(0)), Ok(4));
        assert_eq!(buffer.seek(SeekFrom::Current(-1)), Ok(3));
    }

    #[test]
    fn test_seek_double_bounds() {
        set_example_allocator(4096);
        let mut buffer = AbstractBuffer::new(
            Box::new(TestBuffer::<10>::new()),
            2..=7
        );

        assert_eq!(buffer.seek(SeekFrom::Start(0)), Ok(0));
        assert_eq!(buffer.seek(SeekFrom::End(0)), Ok(5));
        assert_eq!(buffer.seek(SeekFrom::Current(-1)), Ok(4));
    }

}