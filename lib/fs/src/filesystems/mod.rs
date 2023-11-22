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

mod dosfs;
mod fakefs;

#[macro_export]
macro_rules! sub_fsprovider {
    ($type:ty, $prv_sub:tt) => {
        impl crate::io::FileProvider for $type {}

        impl crate::io::Metadata for $type {
            fn len(&self) -> u64 {
                self.$prv_sub.len()
            }

            fn kind(&self) -> crate::io::EntryType {
                self.$prv_sub.kind()
            }

            fn can_read(&self) -> bool {
                self.$prv_sub.can_read()
            }

            fn can_seek(&self) -> bool {
                self.$prv_sub.can_seek()
            }

            fn can_write(&self) -> bool {
                self.$prv_sub.can_write()
            }

            fn permissions(&self) -> crate::permission::Permissions {
                self.$prv_sub.permissions()
            }

            fn date_created(&self) -> Option<crate::UnixTime> {
                self.$prv_sub.date_created()
            }

            fn date_accessed(&self) -> Option<crate::UnixTime> {
                self.$prv_sub.date_accessed()
            }

            fn date_removed(&self) -> Option<crate::UnixTime> {
                self.$prv_sub.date_removed()
            }

            fn date_modified(&self) -> Option<crate::UnixTime> {
                self.$prv_sub.date_modified()
            }
        }

        impl crate::io::Read for $type {
            fn read(&mut self, buf: &mut [u8]) -> crate::FsResult<usize> {
                self.$prv_sub.read(buf)
            }

            fn read_exact(&mut self, buf: &mut [u8]) -> crate::FsResult<()> {
                self.$prv_sub.read_exact(buf)
            }

            fn read_to_end(&mut self, buf: &mut qk_alloc::vec::Vec<u8>) -> crate::FsResult<usize> {
                self.$prv_sub.read_to_end(buf)
            }

            fn read_vectored(&mut self, buf: &mut [&mut [u8]]) -> crate::FsResult<usize> {
                self.$prv_sub.read_vectored(buf)
            }

            fn read_to_string(
                &mut self,
                buf: &mut qk_alloc::string::String,
            ) -> crate::FsResult<usize> {
                self.$prv_sub.read_to_string(buf)
            }

            fn is_read_vectored(&self) -> bool {
                self.$prv_sub.is_read_vectored()
            }
        }

        impl crate::io::Write for $type {
            fn write(&mut self, buf: &[u8]) -> crate::FsResult<usize> {
                self.$prv_sub.write(buf)
            }

            fn flush(&mut self) -> crate::FsResult<()> {
                self.$prv_sub.flush()
            }

            fn write_all(&mut self, buf: &[u8]) -> crate::FsResult<()> {
                self.$prv_sub.write_all(buf)
            }

            fn write_fmt(&mut self, fmt: core::fmt::Arguments<'_>) -> crate::FsResult<()> {
                self.$prv_sub.write_fmt(fmt)
            }

            fn write_vectored(&mut self, bufs: &[&[u8]]) -> crate::FsResult<usize> {
                self.$prv_sub.write_vectored(bufs)
            }

            fn is_write_vectored(&self) -> bool {
                self.$prv_sub.is_write_vectored()
            }
        }

        impl crate::io::Seek for $type {
            fn seek(&mut self, pos: crate::io::SeekFrom) -> crate::FsResult<u64> {
                self.$prv_sub.seek(pos)
            }

            fn rewind(&mut self) -> crate::FsResult<()> {
                self.$prv_sub.rewind()
            }

            fn stream_len(&mut self) -> crate::FsResult<u64> {
                self.$prv_sub.stream_len()
            }

            fn stream_len_dirty(&mut self) -> crate::FsResult<u64> {
                self.$prv_sub.stream_len_dirty()
            }

            fn stream_position(&mut self) -> crate::FsResult<u64> {
                self.$prv_sub.stream_position()
            }
        }
    };
}
