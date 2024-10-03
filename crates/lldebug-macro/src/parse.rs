use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, Error, Expr, Lit, Result, Type,
};

pub struct DebugStream {
    doc_string: Vec<String>,
    debug_type: Type,
    init_expr: Expr,
}

mod reserved {
    syn::custom_keyword!(Debug);
}

impl Parse for DebugStream {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<reserved::Debug>()?;
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
            }
        }
        todo!()
    }
}
