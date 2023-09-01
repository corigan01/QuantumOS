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

use crate::FsResult;
use crate::io::SeekFrom;

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> FsResult<usize>;

    // # Provided
    //fn read_vectored(&mut self, bufs: &mut [])
}

pub trait Write {

}

/// # Seek
/// The `Seek` trait allows one to implement a 'cursor' of sorts to allow
/// movement throughout a stream of bytes. Usually `Seek` has a known size,
/// but its not required and can be arbitrary if required.
///
/// ## Examples
/// One example of something that implements `Seek` is the std `File`. Here we will
/// use `std::io::SeekFrom` to demonstrate, but the workings are the same.
/// ```no_run
/// use std::fs::File;
/// use std::io::SeekFrom;
/// use std::io::Seek;
///
/// fn main() {
///     let mut file = File::open("something.txt").unwrap();
///
///     // We move our 'cursor' 10-bytes into the file
///     file.seek(SeekFrom::Start(10)).unwrap();
///
///     // If we read our file here, we would start reading from the 10th byte.
///     // read...
/// }
/// ```
///
pub trait Seek {
    /// # seek
    /// Move the 'cursor' in the stream of bytes. This can be an offset from the start, end,
    /// or current position in the stream.
    ///
    /// ## Ways of moving the Cursor
    ///
    /// #### `SeekFrom::Start(x)`
    /// Will move the cursor relative to the start position of the stream.
    ///
    /// #### `SeekFrom::End(x)`
    /// Will move the cursor relative to the end of the stream.
    ///
    /// #### `SeekFrom::Current(x)`
    /// Will move the cursor relative to the current stream position.
    ///
    /// ## Errors
    /// Just like any other file operation, seeking can fail. This could be due to seeking past
    /// the end of the buffer, or maybe seeking before the start of the buffer. Sometimes
    /// seek might just fail to read from the stream, or flush its buffer. Errors are defined by
    /// implementation.
    ///
    /// #### Expected Error
    /// The expected return error for `seek` is `FsErrorKind::NotSeekable`. If and only if seek
    /// failed to perform another operation like reading from the stream, or flushing its buffer
    /// it should return another error. This is to hopefully be consistent with logic that might
    /// look for `NotSeekable` errors to control other parts of the logic.
    ///
    /// ## Examples
    /// Here we will use the 'std' lib to demonstrate, which our implementation is to be
    /// consistent with.
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::SeekFrom;
    /// use std::io::Seek;
    ///
    /// fn main() {
    ///     let mut file = File::open("something.txt").unwrap();
    ///
    ///     // We move our 'cursor' 10-bytes into the file
    ///     file.seek(SeekFrom::Start(10)).unwrap();
    ///
    ///     // If read our file here, we would start reading from the 10th byte.
    ///     // read...
    /// }
    /// ```
    fn seek(&mut self, pos: SeekFrom) -> FsResult<u64>;

    /// # Rewind
    /// `rewind` is *exactly* defined to `self.seek(SeekFrom::Start(0))`. `rewind` is used
    /// to 'reset' the cursor to the start of the buffer (like rewinding a vhs tape). Usually
    /// this is used to be descriptive in your file for what you are doingt.
    ///
    /// # Example
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::Seek;
    ///
    /// fn main() {
    ///     let mut file = File::open("my-file.txt").unwrap();
    ///
    ///     // -- snip --
    ///     // file's buffer is no longer known, and we might want to reset it.
    ///     file.rewind().unwrap();
    /// }
    /// ```
    #[inline]
    fn rewind(&mut self) -> FsResult<()> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }


    /// # Stream Length
    /// Gets the length of the stream by seeking to the end of the stream, then to prevent the
    /// loss of stream position, it seeks back to the original position. This function uses
    /// 3-calls to seek, and depending on implementation could be costly, so if the current
    /// stream position is not important then you can use `stream_len_dirty` which will
    /// not reset the current stream position.
    ///
    /// # Note
    /// `stream_len` uses `stream_position`, `SeekFrom::Start(x)`, and `SeekFrom::End(0)` to
    /// gather needed information and reset the stream. If implementation requires these
    /// operations to not operate as expected, reimplementing this function is to be required.
    ///
    /// If seeking fails, this function dirties the current cursor position. This means that if
    /// this function returns an error, the current seek position should not be trusted, and
    /// will need to be reset by the caller.
    #[inline]
    fn stream_len(&mut self) -> FsResult<u64> {
        let current = self.stream_position()?;
        let end = self.seek(SeekFrom::End(0))?;
        self.seek(SeekFrom::Start(current))?;

        Ok(end)
    }

    /// # Stream Length Dirty
    /// This function is *exactly* `self.seek(SeekFrom::End(0))`, and will not reset the
    /// current cursor position. It is to be expected for the caller to reset this before
    /// calling another operation (like read, write, or seek).
    ///
    /// # Note
    /// This function is not defined in the `std`, and its implementation here is for
    /// QuantumOS use only.
    #[inline]
    fn stream_len_dirty(&mut self) -> FsResult<u64> {
        self.seek(SeekFrom::End(0))
    }

    /// # Stream Position
    /// This will retrieve the current stream position.
    #[inline]
    fn stream_position(&mut self) -> FsResult<u64> {
        self.seek(SeekFrom::Current(0))
    }
}