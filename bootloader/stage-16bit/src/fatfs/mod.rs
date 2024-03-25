use self::bpb::Bpb;
use crate::io::{Read, Seek};

mod bpb;

pub enum FatKind {
    Fat12,
    Fat16,
    Fat32,
}

pub(super) trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

pub struct Fat<Part: ReadSeek> {
    disk: Part,
    bpb: Bpb,
}

impl<Part: ReadSeek> Fat<Part> {
    pub fn new(mut disk: Part) -> Result<Self, &'static str> {
        let bpb = Bpb::new(&mut disk)?;

        Ok(Self { disk, bpb })
    }
}
