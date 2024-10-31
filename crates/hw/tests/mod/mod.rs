use hw::hw_device;

hw_device! {
    mod test {
        fn read() -> u32 {
            0
        }

        fn write(input: u32) {
            println!("Got Input = {}", input);
        }
    }
}

fn main() {}
