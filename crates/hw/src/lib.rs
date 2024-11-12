#![no_std]

pub use hw_macro::*;

#[make_hw(
  /// First Bit Field
  field(RW, 0..2, first),
  /// Second Bit Field
  field(RW, 2..5, second),
)]
#[derive(Clone, Copy)]
struct ExampleMultiStruct(u8);
