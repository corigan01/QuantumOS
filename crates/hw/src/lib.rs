pub use hw_macro::*;

hw_macro::hw_device! {
    mod cr0 {
        pub fn read() -> u32 {
            0
        }

        pub fn write(input: u32) {
            println!("Value: {}", input);
        }
    }

    #[field(RO, 0, cr0)]
    protected_mode,

    #[field(RO, 8..16, cr0)]
    dingus_mode,
}
