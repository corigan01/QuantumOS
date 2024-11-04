use syn::{
    parenthesized,
    parse::{Parse, Parser},
    punctuated::Punctuated,
    token::Paren,
    Attribute, Expr, ExprRange, Ident, Lit, LitInt, Token, Visibility,
};

#[derive(Debug)]
pub struct MakeHwMacroInput {
    fields: Punctuated<BitField, Token![,]>,
}

mod keywords {
    syn::custom_keyword!(field);
}

#[derive(Debug)]
pub struct BitField {
    attr: Vec<Attribute>,
    keyword: keywords::field,
    paren_token: Paren,
    access: Access,
    bits: Bits,
    vis: Visibility,
    ident: Ident,
}

impl Parse for BitField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr = input.call(Attribute::parse_outer)?;
        let keyword = input.parse()?;

        let content;
        let paren_token = parenthesized!(content in input);
        let access = content.parse()?;
        content.parse::<Token![,]>()?;
        let bits = content.parse()?;
        content.parse::<Token![,]>()?;
        let vis = content.parse()?;
        let ident = content.parse()?;

        Ok(Self {
            attr,
            keyword,
            paren_token,
            access,
            bits,
            vis,
            ident,
        })
    }
}

impl Parse for MakeHwMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fields = input.parse_terminated(BitField::parse, Token![,])?;
        println!("{:#?}", fields);

        Ok(Self { fields })
    }
}

#[derive(Debug)]
pub enum Access {
    RW,
    RO,
    WO,
    RW1C,
    RW1O,
}

mod access {
    syn::custom_keyword!(RW);
    syn::custom_keyword!(RO);
    syn::custom_keyword!(WO);
    syn::custom_keyword!(RW1C);
    syn::custom_keyword!(RW1O);
}

impl Parse for Access {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(access::RW) {
            input.parse::<access::RW>()?;
            Ok(Access::RW)
        } else if lookahead.peek(access::RO) {
            input.parse::<access::RO>()?;
            Ok(Access::RO)
        } else if lookahead.peek(access::WO) {
            input.parse::<access::WO>()?;
            Ok(Access::WO)
        } else if lookahead.peek(access::RW1C) {
            input.parse::<access::RW1C>()?;
            Ok(Access::RW1C)
        } else if lookahead.peek(access::RW1O) {
            input.parse::<access::RW1O>()?;
            Ok(Access::RW1O)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
pub enum Bits {
    Single(LitInt),
    Range(ExprRange),
}

impl Parse for Bits {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr: Expr = input.parse()?;
        match expr {
            Expr::Lit(syn::ExprLit {
                lit: Lit::Int(int), ..
            }) => Ok(Self::Single(int)),
            Expr::Range(range) => Ok(Self::Range(range)),
            _ => Err(input.error("Expected a bit (literal) or bit-range (eg. 1..2 or 5..=10)")),
        }
    }
}
