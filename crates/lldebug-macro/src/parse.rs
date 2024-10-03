use std::fmt::Debug;

use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, Error, Expr, Lit, Result, Token, Type,
};

pub struct DebugStream {
    doc_strings: Vec<String>,
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

        input.parse::<reserved::Debug>()?;
        input.parse::<Token![:]>()?;
        let debug_type: syn::Type = input.parse()?;
        input.parse::<Token![=]>()?;
        let init_expr: syn::Expr = input.parse()?;
        input.parse::<Token![;]>()?;

        Ok(Self {
            doc_strings,
            debug_type,
            init_expr,
        })
    }
}
