#![no_std]

pub use hw_macro::*;

#[make_hw(
    field(RW, 0..2, pub dingus),
)]
#[derive(Copy, Clone)]
struct Dingus(u32);
