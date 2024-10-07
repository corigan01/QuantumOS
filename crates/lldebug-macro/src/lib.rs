/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2024 Gavin Kellam

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

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
    "Serial": Serial = Serial::new();
}


#[debug_ready]
fn main() {
    println!("Hello World"); --> $TYPE.write(...)
}

// Hmmm, maybe if one uses 'Debug' maybe we should disable
//       the extra '[$WHERE->$STREAM]' printing.


--- TERMINAL:
[Stage32->Serial]: Hello World!
[Stage32->Display]: Hello World!
*/

#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod generate;
mod parse;

/// # Make Debug
/// This is a macro!
#[proc_macro]
pub fn make_debug(token_input: TokenStream) -> TokenStream {
    let single_debug_item = parse_macro_input!(token_input as parse::DebugMacroInput);

    quote! {}.into()
}
