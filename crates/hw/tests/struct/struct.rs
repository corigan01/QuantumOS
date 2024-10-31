use hw::hw_device;

hw_device! {
    pub struct StructTest(u32);

    #[field(RW, 0..2, StructTest)]
    pub multi,

    #[field(RW, 10, StructTest)]
    pub single,
}

fn main() {}
