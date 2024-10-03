/*
// make_debug! can be lazy, so the first `println!()` call will
// construct the output streams.
make_debug! {
    #[debug(Serial)]
    fn() -> Option<Serial> { ... }

    #[debug(ScreenBuffer)]
    // This could be in-case there is no Default implemented
    fn() -> Option<ScreenBuffer> { ... }
}

->


mod _debug {
    static mut DEBUG_OUTPUT_STREAM_SERIAL: Mutex<LazyCell<Option<Serial>>>
        = Mutex::new(LazyCell::new(|| { ... }));

    static mut DEBUG_OUTPUT_STREAM_SCREEN_BUFFER: Mutex<LazyCell<Option<ScreenBuffer>>>
        = Mutex::new(LazyCell::new(|| { ... }));

    fn init_macro() {
        lldebug::set_output_fn(GLOBAL_OUTPUT_PTR);
    }

    const GLOBAL_OUTPUT_PTR: fn(fmt::Arguments) -> fmt::Result = global_output;
    fn global_output(fmt: fmt::Arguments) -> fmt::Result {
        // List of all output streams

        /*Make sure is Some(x)*/ DEBUG_OUTPUT_STREAM_SERIAL.write_fmt(fmt)?;
        /*Make sure is Some(x)*/ DEBUG_OUTPUT_STREAM_SCREEN_BUFFER.write_fmt(fmt)?;

        Ok(())
    }
}

init_macro() --> LLDebug
println!() --> LLDebug --> global_output --> Serial -- init -- maybe { fmt::Write }
                                |
                                > ScreenBuffer -- init -- maybe { fmt::Write }

#[debug_ready] --> Will paste `init_macro()` in main before anything else
fn main() {
    // Can we use the proc macro to put this here?
    init_macro();
}

---

make_debug! {
    Debug: Option<$TYPE> = {$EXPR};
    Debug: $TYPE = {$EXPR};
}
*/

#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use syn::parse_macro_input;
mod parse;

#[proc_macro]
pub fn make_debug(token_input: TokenStream) -> TokenStream {
    let single_debug_item = parse_macro_input!(token_input as parse::DebugStream);
    println!("{:#?}", single_debug_item);

    todo!()
}

#[cfg(test)]
mod test {
    #[test]
    fn compile_one_case() {
        let t = trybuild::TestCases::new();
        t.pass("tests/one.rs");
    }
}
