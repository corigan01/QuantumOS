#[derive(Debug, Clone, Copy)]
pub enum FsError {
    EndOfFile,
    ReadError,
    InvalidInput,
    NotFound,
    NotSupported,
}

pub type Result<T> = core::result::Result<T, FsError>;
