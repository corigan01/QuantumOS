#![no_std]

pub use hw_macro::*;

#[make_hw(
    field(RW, 0, pub dingus),
    field(RW, 0, pub dingus),
    field(RW, 0, dingus),
)]
struct Dingus(u32);
