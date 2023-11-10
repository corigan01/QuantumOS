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

use core::fmt::{Debug, Display, Formatter};
use core::mem::size_of;
use qk_alloc::format;
use qk_alloc::string::String;

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq)]
pub enum FsErrorKind {
    /// # Not Found
    /// An entity was not found. This is most often caused by a file.
    NotFound,
    /// # Permission Denied
    /// You do not have the correct permission to open or use this entity.
    PermissionDenied,
    /// # Address In Use
    /// The socket or path could not be bound due to a already existing connection.
    AddrInUse,
    /// # Broken Pipe
    /// The pipe was closed and the operation could not be continued.
    BrokenPipe,
    /// # Already Exists
    /// The file, path, or other entity could be be created because it already exists.
    AlreadyExists,
    /// # Would Block
    /// The operation would need to block the CPU in order to complete its request, however,
    /// it was configured to avoid blocking.
    WouldBlock,
    /// # Not a Directory
    /// The filesystem entity is not, or was not a directory and the operation can not continue.
    /// Often times this can be caused by a directory existing beforehand, but unexpectedly
    /// became a file.
    NotADirectory,
    /// # Is a Directory
    /// The filesystem entity is, or was a directory and the operation can not continue.
    /// Often times this can be caused by a file existing beforehand, but expectedly became
    /// a directory.
    IsADirectory,
    /// # Read Only Filesystem
    /// The current filesystem is read-only, thus the operation can not continue. Often times
    /// this is caused by trying to write to a file, or create a directory.
    ReadOnlyFilesystem,
    /// # Filesystem Loop
    /// Loop in the filesystem making the system unable to resolve it. This can be caused
    /// by having too many symbolic links, or having links point to themselves. The system
    /// will not, or can not resolve this many entries even if there is a finite end.
    FilesystemLoop,
    /// # Invalid Input
    /// The given input parameter was incorrect, null, or otherwise unexpected and thus invalid.
    InvalidInput,
    /// # Invalid Data
    /// The given input data failed checks, contained malformed input, or otherwise could
    /// not be completed. This can be caused from the filesystem having wrong data, bad sectors,
    /// unimplemented features, or simply wrong input.
    InvalidData,
    /// # Timed Out
    /// The operation took longer then the maximum exceeded limit, and will not continue.
    TimedOut,
    /// # Write Zero
    /// The write operation returned `Ok(0)`, which means it wrote zero bytes. Often times
    /// this is caused by the write operation requesting a particular number of bytes, but
    /// receiving more or less bytes causing the operation to be unsuccessful.
    WriteZero,
    /// # Storage Full
    /// The medium for which you are writing can no-longer hold anymore data. This can be caused
    /// from a full disk, or by unexpectedly being unable to write.
    StorageFull,
    /// # Not Seekable
    /// The seek requested was unable to be preformed. Often times the cause will be trying to seek
    /// out of bounds, or trying to access data not allocated or owned.
    NotSeekable,
    /// # Filesystem Quota Exceeded
    /// The amount of some-data or parameter could not be completed due to the quota's limit being
    /// reached.
    FilesystemQuotaExceeded,
    /// # File too large
    /// Either by filesystem design constraints, or software limits, the file has reached the maximum
    /// size and can no longer be extended. If you are creating a new file, the `len` of such file
    /// needs to be shortened for the write to be successful.
    FileTooLarge,
    /// # Resource Busy
    /// The requested resource could not be bothered to take your request, it is very busy at the
    /// moment.
    ResourceBusy,
    /// # Executable File Busy
    /// The executable file is either in-use by another process, or does not like your tone. Please
    /// try again later.
    ExecutableFileBusy,
    /// # Deadlock
    /// A deadlock is when something is running without end and is thus 'locked', but since the
    /// operation was unable to be stopped it is 'dead.'
    Deadlock,
    /// # Crosses Devices
    /// The link, directory, or file crosses devices or filesystems and the operation was
    /// unable to continue.
    CrossesDevices,
    /// # Too Many Links
    /// The amount of links either by filesystem design or software limits has reached the maximum.
    /// No more links can be created.
    TooManyLinks,
    /// # Invalid Filename
    /// Either due to filesystem design or by software limitations, this filename is invalid. This
    /// can be caused by either invalid characters, length (too many), or length (not enough).
    InvalidFilename,
    /// # Argument List Too Long
    /// The program or function input contains too many elements, and can not complete the request.
    ArgumentListTooLong,
    /// # Interrupted
    /// The operation was interrupted, and failed to complete. Often times the operation can be
    /// resumed, or retried.
    Interrupted,
    /// # Unsupported
    /// The operation could not be handled due to a unimplemented feature, or by design constraints
    /// of the filesystem or software. This usually means that the requested operation could never
    /// succeed on this platform or device.
    Unsupported,
    /// # Unexpected EOF (End of File)
    /// While preforming the operation, more data was expected but reached the Eof (end of file).
    /// Often this happens when you attempt to read more bytes then the filesize, causing the end
    /// of the file to appear sooner then the last byte. Attempting to read less bytes could
    /// solve the problem.
    UnexpectedEof,
    /// # Out Of Memory
    /// The system has run out of memory, and the operation can not continue due to
    /// more memory is required to complete.
    OutOfMemory,
    /// # Other
    /// The error is not known.
    Other,
}

impl FsErrorKind {
    /// # As Str
    /// Convert this type into a human readable string. This can be useful when printing, or showing
    /// in UI.
    ///
    /// # Examples
    /// ```rust
    /// use fs::error::FsErrorKind;
    ///
    /// fn main() {
    ///     let not_found_message = FsErrorKind::NotFound.as_str();
    ///     assert_eq!(not_found_message, "not found");
    /// }
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            FsErrorKind::NotFound => "not found",
            FsErrorKind::PermissionDenied => "permission was denied",
            FsErrorKind::AddrInUse => "address already in use",
            FsErrorKind::BrokenPipe => "broken pipe",
            FsErrorKind::AlreadyExists => "item already exists",
            FsErrorKind::WouldBlock => "request would block",
            FsErrorKind::NotADirectory => "not a Directory",
            FsErrorKind::IsADirectory => "is a directory",
            FsErrorKind::ReadOnlyFilesystem => "read only filesystem",
            FsErrorKind::FilesystemLoop => "loop In filesystem",
            FsErrorKind::InvalidInput => "invalid input",
            FsErrorKind::InvalidData => "invalid data",
            FsErrorKind::TimedOut => "timed out",
            FsErrorKind::WriteZero => "wrote zero bytes",
            FsErrorKind::StorageFull => "storage is full",
            FsErrorKind::NotSeekable => "not seekable",
            FsErrorKind::FilesystemQuotaExceeded => "filesystem quota was exceeded",
            FsErrorKind::FileTooLarge => "file too large",
            FsErrorKind::ResourceBusy => "resource is busy",
            FsErrorKind::ExecutableFileBusy => "executable is busy",
            FsErrorKind::Deadlock => "deadlock",
            FsErrorKind::CrossesDevices => "crosses devices",
            FsErrorKind::TooManyLinks => "too many links",
            FsErrorKind::InvalidFilename => "invalid filename",
            FsErrorKind::ArgumentListTooLong => "argument list is too large",
            FsErrorKind::Interrupted => "interrupted",
            FsErrorKind::Unsupported => "unsupported request",
            FsErrorKind::UnexpectedEof => "unexpected EOF",
            FsErrorKind::OutOfMemory => "out of memory",
            _ => "Unknown or Unimplemented Case",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            FsErrorKind::NotFound => "An entity was not found. This is most often caused by a file.",
            FsErrorKind::PermissionDenied => "You do not have the correct permission to open or use this entity.",
            FsErrorKind::AddrInUse => "The socket or path could not be bound due to a already existing connection.",
            FsErrorKind::BrokenPipe => "The pipe was closed and the operation could not be continued.",
            FsErrorKind::AlreadyExists => "The file, path, or other entity could be be created because it already exists.",
            FsErrorKind::WouldBlock => "The operation would need to block the CPU in order to complete its request, however, it was configured to avoid blocking.",
            FsErrorKind::NotADirectory => "The filesystem entity is not, or was not a directory and the operation can not continue. Often times this can be caused by a directory existing beforehand, but unexpectedly became a file.",
            FsErrorKind::IsADirectory => "The filesystem entity is, or was a directory and the operation can not continue. Often times this can be caused by a file existing beforehand, but expectedly became a directory.",
            FsErrorKind::ReadOnlyFilesystem => "The current filesystem is read-only, thus the operation can not continue. Often times this is caused by trying to write to a file, or create a directory.",
            FsErrorKind::FilesystemLoop => "Loop in the filesystem making the system unable to resolve it. This can be caused by having too many symbolic links, or having links point to themselves. The system will not, or can not resolve this many entries even if there is a finite end.",
            FsErrorKind::InvalidInput => "The given input parameter was incorrect, null, or otherwise unexpected and thus invalid.",
            FsErrorKind::InvalidData => "The given input data failed checks, contained malformed input, or otherwise could not be completed. This can be caused from the filesystem having wrong data, bad sectors, unimplemented features, or simply wrong input.",
            FsErrorKind::TimedOut => "The operation took longer then the maximum exceeded limit, and will not continue.",
            FsErrorKind::WriteZero => "The write operation returned `Ok(0)`, which means it wrote zero bytes. Often times this is caused by the write operation requesting a particular number of bytes, but receiving more or less bytes causing the operation to be unsuccessful.",
            FsErrorKind::StorageFull => "The medium for which you are writing can no-longer hold anymore data. This can be caused from a full disk, or by unexpectedly being unable to write.",
            FsErrorKind::NotSeekable => "The seek requested was unable to be preformed. Often times the cause will be trying to seek out of bounds, or trying to access data not allocated or owned.",
            FsErrorKind::FilesystemQuotaExceeded => "The amount of some-data or parameter could not be completed due to the quota's limit being reached.",
            FsErrorKind::FileTooLarge => "Either by filesystem design constraints, or software limits, the file has reached the maximum size and can no longer be extended. If you are creating a new file, the `len` of such file needs to be shortened for the write to be successful.",
            FsErrorKind::ResourceBusy => "The requested resource could not be bothered to take your request, it is very busy at the moment.",
            FsErrorKind::ExecutableFileBusy => "The executable file is either in-use by another process, or does not like your tone. Please try again later.",
            FsErrorKind::Deadlock => "A deadlock is when something is running without end and is thus 'locked', but since the operation was unable to be stopped it is 'dead.'",
            FsErrorKind::CrossesDevices => "The link, directory, or file crosses devices or filesystems and the operation was unable to continue.",
            FsErrorKind::TooManyLinks => "The amount of links either by filesystem design or software limits has reached the maximum. No more links can be created.",
            FsErrorKind::InvalidFilename => "Either due to filesystem design or by software limitations, this filename is invalid. This can be caused by either invalid characters, length (too many), or length (not enough).",
            FsErrorKind::ArgumentListTooLong => "The program or function input contains too many elements, and can not complete the request.",
            FsErrorKind::Interrupted => "The operation was interrupted, and failed to complete. Often times the operation can be resumed, or retried.",
            FsErrorKind::Unsupported => "The operation could not be handled due to a unimplemented feature, or by design constraints of the filesystem or software. This usually means that the requested operation could never succeed on this platform or device.",
            FsErrorKind::UnexpectedEof => "While preforming the operation, more data was expected but reached the Eof (end of file). Often this happens when you attempt to read more bytes then the filesize, causing the end of the file to appear sooner then the last byte. Attempting to read less bytes could solve the problem.",
            FsErrorKind::OutOfMemory => "The system has run out of memory, and the operation can not continue due to more memory is required to complete.",
            _ => "The error is not known.",
        }
    }
}

impl Display for FsErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}\n{}", self.as_str(), self.description()))
    }
}

impl Debug for FsErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}", self.as_str()))
    }
}

impl Display for FsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#?}", self))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FsError {
    error: String,
    kind: FsErrorKind,
}

impl FsError {
    pub fn new(kind: FsErrorKind, error: &str) -> FsError {
        Self {
            error: String::from(error),
            kind,
        }
    }

    pub fn from_string(kind: FsErrorKind, error: String) -> FsError {
        Self { error, kind }
    }

    pub fn try_from_array_error<Type>(array: &[u8]) -> FsError {
        let error = format!(
            concat!(
                "Cannot construct ",
                stringify!(Type),
                " with improperly sized array!\n",
                "\tArray only has {} bytes, but expected {} bytes!"
            ),
            array.len(),
            size_of::<Type>()
        );

        Self {
            error,
            kind: FsErrorKind::InvalidInput,
        }
    }

    pub fn other(error: &str) -> FsError {
        Self {
            error: String::from(error),
            kind: FsErrorKind::Other,
        }
    }

    pub fn into_inner(self) -> String {
        self.error
    }

    pub fn kind(&self) -> FsErrorKind {
        self.kind
    }
}

impl From<FsErrorKind> for FsError {
    fn from(value: FsErrorKind) -> Self {
        Self {
            error: String::new(),
            kind: value,
        }
    }
}
