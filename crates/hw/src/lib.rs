#![no_std]

pub use hw_macro::*;

#[make_hw(
    field(RW, 0, pub dingus),
)]
struct Dingus(u32);
