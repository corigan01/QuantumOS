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

use proc_macro2::Span;
use std::fmt::Debug;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, Error, Expr, Lit, LitStr, Result, Token, Type,
};

pub struct DebugStream {
    doc_strings: Vec<String>,
    stream_name: Option<LitStr>,
    debug_type: Type,
    init_expr: Expr,
    stream_span: Span,
}

impl Debug for DebugStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugStream")
            .field("doc_strings", &self.doc_strings)
            .finish()
    }
}

mod reserved {
    syn::custom_keyword!(Debug);
}

impl Parse for DebugStream {
    fn parse(input: ParseStream) -> Result<Self> {
        let stream_span = input.span();
        let attributes = input.call(Attribute::parse_outer)?;
        let mut doc_strings = Vec::new();

        for attribute in &attributes {
            if attribute.path().is_ident("doc") {
                let Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(ref doc_string),
                    attrs: _,
                }) = attribute.meta.require_name_value()?.value
                else {
                    attribute
                        .span()
                        .unwrap()
                        .error("doc attribute must contain string expression")
                        .help("doc attributes should follow this standard: #[doc = \"Doc String Message\"]")
                        .emit();

                    return Err(Error::new(
                        attribute.span(),
                        "Failed to parse doc attribute",
                    ));
                };

                doc_strings.push(doc_string.value());
            } else {
                return Err(Error::new(
                    attribute.span(),
                    format!(
                        "Attribute '{}' is unknown!",
                        attribute.path().require_ident()?
                    ),
                ));
            }
        }

        let stream_name = match input.parse::<LitStr>() {
            Ok(str) => Some(str),
            Err(_) => {
                input.parse::<reserved::Debug>()?;
                None
            }
        };

        input.parse::<Token![:]>()?;
        let debug_type: syn::Type = input.parse()?;
        input.parse::<Token![=]>()?;
        let init_expr: syn::Expr = input.parse()?;

        Ok(Self {
            doc_strings,
            stream_name,
            debug_type,
            init_expr,
            stream_span,
        })
    }
}

pub struct DebugMacroInput {
    streams: Vec<DebugStream>,
    macro_span: Span,
}

impl Parse for DebugMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_span = input.span();
        let streams = input
            .parse_terminated(DebugStream::parse, Token![;])?
            .into_iter()
            .collect();

        Ok(Self {
            streams,
            macro_span,
        })
    }
}
