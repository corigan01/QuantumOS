use crate::error::Result;

#[allow(dead_code)]
pub trait Seek {
    fn seek(&mut self, pos: u64) -> u64;
    fn stream_position(&mut self) -> u64;
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}
