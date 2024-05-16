#[derive(Clone, Copy, Debug)]
pub enum BootloaderError {
    DiskError,
    EOF,
    BiosError,
    NotFound,
    InvalidInput,
}

pub type Result<T> = core::result::Result<T, BootloaderError>;
