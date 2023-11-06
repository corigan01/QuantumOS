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

use core::fmt::Arguments;

use qk_alloc::string::String;
use qk_alloc::vec::Vec;

use crate::error::{FsError, FsErrorKind};
use crate::io::SeekFrom;
use crate::FsResult;

pub trait ReadWriteSeek: Read + Write + Seek {}
impl<T: Read + Write + Seek> ReadWriteSeek for T {}

/// # Read
/// Able to read bytes from a stream. Structs that implement `read` are called 'readers' (according
/// to the rust std lib). Read should advance the read cursor.
pub trait Read {
    /// # Read
    /// implementation of read to be able to read from a stream.
    fn read(&mut self, buf: &mut [u8]) -> FsResult<usize>;

    /// # Read Vectored
    /// Accelerated reading of buffers-of-buffers. Should be identical to sending one read call to
    /// one large buffer.
    fn read_vectored(&mut self, buf: &mut [&mut [u8]]) -> FsResult<usize> {
        todo!("Not Implemented read_vectored");
    }

    /// # Is Read Vectored?
    /// Checks if reading from the stream has implemented `read_vectored`. If this stream does not
    /// contain support for reading vectored, its recommened that one should not use
    /// `read_vectored` as it might be really slow for some cases.
    fn is_read_vectored(&self) -> bool {
        false
    }

    /// # Read to End
    /// Reads to the end of the stream populating your `buf` element with each byte.
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> FsResult<usize> {
        let mut readable_slice = [0_u8; 512];
        let mut total_read = 0;

        loop {
            match self.read(&mut readable_slice) {
                Ok(0) => break,
                Ok(read_amount) => {
                    buf.extend_from_slice(&readable_slice[..read_amount]);
                    readable_slice.iter_mut().for_each(|value| *value = 0);
                    total_read += read_amount;
                }
                Err(kind) if kind.kind() == FsErrorKind::Interrupted => continue,
                Err(other) => return Err(other),
            }
        }

        Ok(total_read)
    }

    /// # Read to String
    /// Reads to the end of the stream populating your `buf` element with each char. If the string
    /// is invalid, an error type `InvalidData` will be returned and `buf` will be unchanged.
    fn read_to_string(&mut self, buf: &mut String) -> FsResult<usize> {
        let mut new_vec = Vec::new();
        let total_read = self.read_to_end(&mut new_vec)?;

        let raw_str = core::str::from_utf8(new_vec.as_slice()).map_err(|_| {
            FsError::new(
                FsErrorKind::InvalidData,
                "read_to_string read invalid utf-8 char in buffer stream",
            )
        })?;

        buf.push_str(raw_str);
        Ok(total_read)
    }

    /// # Read Exact
    /// Attempts to read an exact amount of bytes from the stream. If the stream is smaller then
    /// the amount of bytes in buf `UnexpectedEof` will be returned. Any errors while reading will
    /// return that error, and `buf` could be clobbered.
    fn read_exact(&mut self, buf: &mut [u8]) -> FsResult<()> {
        let mut total_read = 0;
        loop {
            if total_read == buf.len() {
                break;
            }

            match self.read(&mut buf[total_read..]) {
                Ok(0) => return Err(FsError::new(
                    FsErrorKind::UnexpectedEof,
                    "Reached EOF, but more bytes where to be read",
                )),
                Ok(bytes) => total_read += bytes,
                Err(kind) if kind.kind() == FsErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }

        Ok(())
    }

    /// # By Ref
    /// Gets a mutable refrence to the underling type.
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

pub trait Write {
    /// # Write
    /// Writes bytes to a stream. Writting, just like reading should add to the Seek pos. 
    fn write(&mut self, buf: &[u8]) -> FsResult<usize>;

    /// # Flush
    /// Flushs the bytes from any buffers to the final destination. 
    fn flush(&mut self) -> FsResult<()>;

    fn write_vectored(&mut self, bufs: &[&[u8]]) -> FsResult<usize> {
        todo!("Write Vectored")
    }

    fn is_write_vectored(&self) -> bool {
        false
    }

    fn write_all(&mut self, buf: &[u8]) -> FsResult<()> {
        let mut total_written = 0;
        loop {
            if total_written == buf.len() {
                break;
            }

            match self.write(&buf[total_written..]) {
                Ok(0) => {
                    return Err(
                        FsError::new(
                            FsErrorKind::UnexpectedEof, 
                            "Writting all bytes reached the end of the stream, and was not able to write anymore bytes.")) 
                }
                Ok(bytes) => total_written += bytes,
                Err(kind) if kind.kind() == FsErrorKind::Interrupted => continue,
                Err(err) => return Err(err)
            }
        }

        Ok(())
    }

    fn write_fmt(&mut self, fmt: Arguments<'_>) -> FsResult<()> {
        todo!()
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
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

