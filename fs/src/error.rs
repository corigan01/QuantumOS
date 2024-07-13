pub enum FsError {
    EndOfFile,
    ReadError,
    InvalidInput,
    NotFound,
}

pub type Result<T> = core::result::Result<T, FsError>;
