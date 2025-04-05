/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_macro_input, Error, ItemFn};

mod generate;
mod parse;

/// # Make Debug
/// This is a macro!
#[proc_macro]
pub fn make_debug(token_input: TokenStream) -> TokenStream {
    let macro_input = parse_macro_input!(token_input as parse::DebugMacroInput);
    generate::generate_make_debug(macro_input).unwrap().into()
}

#[proc_macro_attribute]
pub fn debug_ready(attr: TokenStream, item: TokenStream) -> TokenStream {
    if attr.is_empty() {
        let macro_input = parse_macro_input!(item as ItemFn);
        generate::generate_debug_ready(macro_input)
    } else {
        Error::new(
            Span::call_site(),
            "Did not expect any arguments in attribute!",
        )
        .into_compile_error()
    }
    .into()
}
