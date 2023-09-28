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
    /// # Readable Range
    /// This is the range in the buffer at-which we are allowed to read and write to. When
    /// seeking in this datatype, SeekFrom::Start(0) will seek `data` from readable_range.start
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

    fn readable_range_begin(&self, buffer_end: u64) -> u64 {
        match self.range_start {
            Bound::Included(value) => min(value, buffer_end),
            Bound::Unbounded => 0,
            _ => unreachable!()
        }
    }

    fn readable_range_end(&self, buffer_end: u64) -> u64 {
        let readable_range = match self.range_end {
            Bound::Included(value) => value,
            Bound::Excluded(value) => value - 1,
            Bound::Unbounded => buffer_end
        };

        min(readable_range, buffer_end)
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

    /// TODO : We want to shrink a range,
    /// e.g:
    ///
    /// Say we have the starting range `..=10`.
    /// And we want to shrink that range with `2..5`
    /// we would then have a range of `2..5`.
    ///
    /// Say we have the starting range `2..13`
    /// and we want to shrink that range with `2..5`
    /// we would then have a range of `4..6`
    const fn range_add(lhs: Bound<u64>, rhs: Bound<u64>) -> Bound<u64> {
        match rhs {
            Bound::Included(inner_val) => {
                match lhs {
                    Bound::Included(outer_val) => Bound::Included(outer_val + inner_val),
                    Bound::Excluded(outer_val) => Bound::Included((outer_val - 1) + inner_val),
                    Bound::Unbounded => rhs,
                }
            }
            Bound::Excluded(inner_val) => {
                match lhs {
                    Bound::Included(outer_val) => Bound::Excluded(outer_val + inner_val),
                    Bound::Excluded(outer_val) => Bound::Excluded((outer_val - 1) + inner_val),
                    Bound::Unbounded => rhs,
                }
            }
            Bound::Unbounded => lhs,
        }
    }

    fn range_sub(lhs: Bound<u64>, rhs: Bound<u64>) -> Bound<u64> {
        todo!()
    }

    pub fn temporary_shrink<Func, Range, AnyOut>(&mut self, tmp_range: Range, mut func: Func) -> AnyOut
        where Func: FnMut(&mut Self) -> AnyOut,
              Range: RangeBounds<u64> {

        let current_begin = self.range_start;
        let current_end = self.range_end;

        let return_result = func(self);

        self.range_start = current_begin;
        self.range_end = current_end;

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

    #[test]
    fn test_range_add() {
        assert_eq!(AbstractBuffer::range_add(Bound::Included(2), Bound::Unbounded), Bound::Included(2));
        assert_eq!(AbstractBuffer::range_add(Bound::Included(2), Bound::Included(1)), Bound::Included(3));
        assert_eq!(AbstractBuffer::range_add(Bound::Unbounded, Bound::Unbounded), Bound::Unbounded);
    }

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