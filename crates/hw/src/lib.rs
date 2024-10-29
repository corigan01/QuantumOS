pub use hw_macro::*;

hw_macro::hw_device! {
    mod test {
        fn write(input: u32) {
            println!("Input = {}", input);
        }

        fn read() -> u32 {
            100
        }
    }
}
