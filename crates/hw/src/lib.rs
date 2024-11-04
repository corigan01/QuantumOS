#![no_std]

pub use hw_macro::*;

#[make_hw(
    /// This is a dingus
    field(RW, 0, pub dingus),
    /// Idk what this is
    field(RW, 0, pub dingus),
    field(RW, 0, dingus),
)]
struct Dingus(u32);
