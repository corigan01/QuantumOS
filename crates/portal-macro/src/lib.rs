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

use portal_parse::PortalMacroInput;
use proc_macro::TokenStream;
use proc_macro_error::{abort_if_dirty, proc_macro_error};
use quote::quote;
use syn::parse_macro_input;
use type_serde::generate_ast_portal;

mod ast;
mod parse;
mod portal_parse;
mod type_serde;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn portal(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as portal_parse::PortalArgs);
    let trait_input = parse_macro_input!(input as portal_parse::PortalTrait);
    let portal_macro = PortalMacroInput { args, trait_input };

    abort_if_dirty();
    generate_ast_portal(&portal_macro).into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn portal2(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as ast::PortalMacroArgs);
    let mut trait_input = parse_macro_input!(input as ast::PortalMacro);
    trait_input.args = Some(args);

    // Let types expore where they are used
    trait_input.type_explore();

    println!("{:#?}", trait_input);

    abort_if_dirty();
    quote! {}.into()
}
