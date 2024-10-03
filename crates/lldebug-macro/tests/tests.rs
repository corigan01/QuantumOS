mod test {
    #[test]
    fn compile_one_case() {
        let t = trybuild::TestCases::new();
        t.pass("tests/one/*.rs");
    }
}
