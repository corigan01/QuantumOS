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

#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use provider_parse::{MacroImplWho, MacroMod, MacroStruct};
use syn::{parse_macro_input, visit::Visit};

mod macro_gen;
pub(crate) mod make_hw_parse;
pub(crate) mod provider_parse;

#[proc_macro_attribute]
pub fn bits(args: TokenStream, input: TokenStream) -> TokenStream {
    let macro_fields = parse_macro_input!(args as make_hw_parse::MakeHwMacroInput);

    // Parse the Struct/Module
    let mut self_visitor = MacroImplWho::Nothing;
    let parsed = syn::parse(input).unwrap();
    self_visitor.visit_file(&parsed);

    match self_visitor {
        MacroImplWho::Nothing => {
            Span::call_site()
                .unwrap()
                .error("Macro can only be applied to struct or mod")
                .emit();

            quote::quote! {}
        }
        MacroImplWho::IsStruct(struct_inner) => {
            let macro_struct = MacroStruct {
                struct_inner,
                macro_fields,
            };

            macro_gen::gen_struct(macro_struct)
        }
        MacroImplWho::IsMod(mod_inner) => {
            let macro_mod = MacroMod {
                mod_inner,
                macro_fields,
            };

            macro_gen::gen_mod(macro_mod)
        }
    }
    .into()
}
