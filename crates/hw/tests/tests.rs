mod test {
    // #[test]
    // fn compile_one_case() {
    //     let t = trybuild::TestCases::new();
    //     t.pass("tests/one/*.rs");
    // }

    // #[test]
    // fn compile_module_case() {
    //     let t = trybuild::TestCases::new();
    //     t.pass("tests/mod/*.rs");
    // }

    // #[test]
    // fn compile_struct_case() {
    //     let t = trybuild::TestCases::new();
    //     t.pass("tests/struct/*.rs");
    // }

    #[test]
    fn ensure_single_bit_set() {
        use hw::make_hw;

        #[make_hw(
            /// First Bit Field
            field(RW, 0, first_bit),
            /// Second Bit Field
            field(RW, 1, second_bit),
        )]
        #[derive(Clone, Copy)]
        struct ExampleStruct(u8);

        let mut ex = ExampleStruct(0);

        assert_eq!(ex.0, 0);
        assert_eq!(ex.is_first_bit_set(), false);
        assert_eq!(ex.is_second_bit_set(), false);

        // Manually set the two flags
        ex.0 = 0b11;

        assert_eq!(ex.is_first_bit_set(), true);
        assert_eq!(ex.is_second_bit_set(), true);

        // Set first bit
        ex.0 = 0;
        ex.set_first_bit_flag(true);
        assert_eq!(ex.0, 0b01);
        assert_eq!(ex.is_first_bit_set(), true);
        assert_eq!(ex.is_second_bit_set(), false);

        // Set second bit
        ex.0 = 0;
        ex.set_second_bit_flag(true);
        assert_eq!(ex.0, 0b10);
        assert_eq!(ex.is_first_bit_set(), false);
        assert_eq!(ex.is_second_bit_set(), true);

        // Unset first bit, making zero
        ex.0 = 0b01;
        ex.set_first_bit_flag(false);
        assert_eq!(ex.0, 0b00);
        assert_eq!(ex.is_first_bit_set(), false);
        assert_eq!(ex.is_second_bit_set(), false);

        // Unset first bit, leaving second bit
        ex.0 = 0b11;
        ex.set_first_bit_flag(false);
        assert_eq!(ex.0, 0b10);
        assert_eq!(ex.is_first_bit_set(), false);
        assert_eq!(ex.is_second_bit_set(), true);

        // Unset second bit, making zero
        ex.0 = 0b10;
        ex.set_second_bit_flag(false);
        assert_eq!(ex.0, 0b00);
        assert_eq!(ex.is_first_bit_set(), false);
        assert_eq!(ex.is_second_bit_set(), false);

        // Unset second bit, leaving first bit
        ex.0 = 0b11;
        ex.set_second_bit_flag(false);
        assert_eq!(ex.0, 0b01);
        assert_eq!(ex.is_first_bit_set(), true);
        assert_eq!(ex.is_second_bit_set(), false);
    }
}
