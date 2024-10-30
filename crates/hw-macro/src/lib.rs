#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use syn::parse_macro_input;

pub(crate) mod hw_gen;
pub(crate) mod hw_parse;

#[proc_macro]
pub fn hw_device(tokens: TokenStream) -> TokenStream {
    let macro_input = parse_macro_input!(tokens as hw_parse::HwDeviceMacro);

    hw_gen::gen(macro_input).into()
}
