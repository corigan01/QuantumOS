use hw::hw_device;

hw_device! {
    mod test {
        pub unsafe fn read() -> u32 {
            todo!()
        }

        pub unsafe fn write(_value: u32) {
            todo!()
        }
    }


    #[field(RW, 12, test)]
    pub dingus,
}

fn main() {}
