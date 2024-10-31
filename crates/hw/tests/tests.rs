mod test {
    #[test]
    fn compile_one_case() {
        let t = trybuild::TestCases::new();
        t.pass("tests/one/*.rs");
    }

    #[test]
    fn compile_module_case() {
        let t = trybuild::TestCases::new();
        t.pass("tests/mod/*.rs");
    }

    #[test]
    fn compile_struct_case() {
        let t = trybuild::TestCases::new();
        t.pass("tests/struct/*.rs");
    }
}
