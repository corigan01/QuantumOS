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

mod info;
mod traits;
pub use info::*;
pub use traits::*;

/// # Entry Type
/// The type of filesystem entry this type is. Determines if its a file, directory, or other kind
/// of entry.
///
/// ### EXT2 Impl
/// - 0x1000 	FIFO
/// - 0x2000 	Character device
/// - 0x4000 	Directory
/// - 0x6000 	Block device
/// - 0x8000 	Regular file
/// - 0xA000 	Symbolic link
/// - 0xC000 	Unix socket
#[repr(u8)]
pub enum EntryType {
    /// # FIFO - First-In First-Out special file (named pipe)
    /// ```text
    /// # Credit Linux Man Page
    ///
    /// A FIFO special file (a named pipe) is similar to a pipe, except
    /// that it is accessed as part of the filesystem.  It can be opened
    /// by multiple processes for reading or writing.  When processes are
    /// exchanging data via the FIFO, the kernel passes all data
    /// internally without writing it to the filesystem.  Thus, the FIFO
    /// special file has no contents on the filesystem; the filesystem
    /// entry merely serves as a reference point so that processes can
    /// access the pipe using a name in the filesystem.
    ///
    /// The kernel maintains exactly one pipe object for each FIFO
    /// special file that is opened by at least one process.  The FIFO
    /// must be opened on both ends (reading and writing) before data can
    /// be passed.  Normally, opening the FIFO blocks until the other end
    /// is opened also.
    ///
    /// A process can open a FIFO in nonblocking mode.  In this case,
    /// opening for read-only succeeds even if no one has opened on the
    /// write side yet and opening for write-only fails with ENXIO (no
    /// such device or address) unless the other end has already been
    /// opened.
    ///
    /// Under Linux, opening a FIFO for read and write will succeed both
    /// in blocking and nonblocking mode.  POSIX leaves this behavior
    /// undefined.  This can be used to open a FIFO for writing while
    /// there are no readers available.  A process that uses both ends of
    /// the connection in order to communicate with itself should be very
    /// careful to avoid deadlocks.
    /// ```
    FIFO = 0x10,
    /// # Character Device
    /// A non-buffered device. Unlike Block Devices, Character Devices are not to be buffered,
    /// and do not have to be random access. Character Devices cannot mount a filesystem.
    ///
    /// TODO: Include Unix documentation link, or ref
    CharacterDevice = 0x20,
    /// # Directory
    /// A entry which contains paths, or links, to one or more other entries. A directory is often
    /// called a folder for which items are stored, and in this case its no different.
    ///
    /// TODO: Include Unix documentation link, or ref
    Directory = 0x40,
    /// # Block Device
    /// Unlike Character Devices, Block Devices are buffered. Commonly Block devices are for disk
    /// drives or other types of devices which need to be buffered. Block devices are reqired to be
    /// random access.
    ///
    /// TODO: Include Unix documentation link, or ref
    BlockDevice = 0x60,
    /// # Regular File
    /// A file which can be either readable, writable, and or seekable. Most files are of this
    /// type, and are the default entry type.
    ///
    /// TODO: Include Unix documentation link, or ref
    File = 0x80,
    /// # Symbolic Link
    /// A link to a directory, or file. Very similar in function to a ptr. It points to another
    /// entry.
    ///
    /// TODO: Include Unix documentation link, or ref
    SymbolicLink = 0xA0,
    /// # Unix Socket
    /// A Unix Socket.
    ///
    /// TODO: Include Unix documentation link, or ref
    UnixSocket = 0xC0,
}

/// # Seek From
/// Seek options for seeking within a stream.
///
/// TODO: document the types
pub enum SeekFrom {
    Start(u64),
    Current(i64),
    End(i64),
}
