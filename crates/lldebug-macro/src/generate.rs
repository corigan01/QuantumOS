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

use crate::parse::{DebugMacroInput, DebugStream};
use quote::quote_spanned;
use syn::{spanned::Spanned, Result};

pub fn generate(macro_input: DebugMacroInput) -> Result<proc_macro2::TokenStream> {
    let each_macro: Vec<proc_macro2::TokenStream> = macro_input
        .streams
        .iter()
        .map(|stream| generate_stream(stream))
        .collect();

    let init_function = generate_init_function(&macro_input);

    Ok(quote_spanned! {macro_input.macro_span=>
        #[doc = "# Debug Macro Mod"]
        mod debug_macro {
            #init_function

            #(#each_macro)*
        }
    })
}

pub fn static_stream_var_name(enumeration: usize, stream: &DebugStream) -> String {
    let stream_name: String = stream
        .stream_name
        .as_ref()
        .map(|stream_name| {
            stream_name
                .value()
                .chars()
                .into_iter()
                .map(|c| match c {
                    ' ' => '_',
                    any => any,
                })
                .filter(char::is_ascii_alphanumeric)
                .map(|l| l.to_ascii_uppercase())
                .collect()
        })
        .unwrap_or_else(|| format!("ANONYMOUS_{enumeration}"));

    format!("DEBUG_STREAM_OUTPUT_{stream_name}")
}

pub fn static_stream_var(name: &str, stream: &DebugStream) -> proc_macro2::TokenStream {
    let stream_type = &stream.debug_type;

    quote_spanned! {stream_type.span()=>
        static #name: core::sync::Mutex<core::cell::LazyCell<#stream_type>>
    }
}

pub fn generate_stream(stream: &DebugStream) -> proc_macro2::TokenStream {
    quote_spanned! {stream.stream_span=>

    }
}

pub fn generate_init_function(macro_input: &DebugMacroInput) -> proc_macro2::TokenStream {
    todo!()
}
