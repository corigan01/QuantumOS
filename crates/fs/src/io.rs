use crate::error::Result;

pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

#[allow(dead_code)]
pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
    fn stream_position(&mut self) -> u64;
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}
