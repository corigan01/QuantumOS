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
        })
    }
}

pub struct DebugMacroInput {
    streams: Vec<DebugStream>,
}

impl Parse for DebugMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let streams = input
            .parse_terminated(DebugStream::parse, Token![;])?
            .into_iter()
            .collect();

        Ok(Self { streams })
    }
}
