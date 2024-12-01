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

use std::ops::Bound;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parenthesized, parse::Parse, punctuated::Punctuated, token::Paren, Attribute, Expr, ExprLit,
    ExprRange, Ident, Lit, LitInt, Token, Type, Visibility,
};

#[derive(Debug)]
pub struct MakeHwMacroInput {
    pub(crate) fields: Punctuated<BitField, Token![,]>,
}

mod keywords {
    syn::custom_keyword!(field);
}

#[derive(Debug)]
pub struct BitField {
    pub(crate) attr: Vec<Attribute>,
    pub(crate) keyword: keywords::field,
    pub(crate) paren_token: Paren,
    pub(crate) access: Access,
    pub(crate) bits: Bits,
    pub(crate) vis: Visibility,
    pub(crate) ident: Ident,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BitFieldType {
    TypeBool,

    Type8,
    Type16,
    Type32,
    Type64,

    InvalidType { start: usize, end: usize },
}

impl Into<usize> for BitFieldType {
    fn into(self) -> usize {
        match self {
            Self::TypeBool => 1,
            Self::Type8 => 8,
            Self::Type16 => 16,
            Self::Type32 => 32,
            Self::Type64 => 64,
            _ => 0,
        }
    }
}

impl Into<TokenStream> for BitFieldType {
    fn into(self) -> TokenStream {
        match self {
            BitFieldType::TypeBool => quote! {bool},
            BitFieldType::Type8 => quote! {u8},
            BitFieldType::Type16 => quote! {u16},
            BitFieldType::Type32 => quote! {u32},
            BitFieldType::Type64 => quote! {u64},
            BitFieldType::InvalidType { .. } => quote! {_},
        }
    }
}

impl Into<BitFieldType> for Type {
    fn into(self) -> BitFieldType {
        match self {
            Type::Path(type_path) => match () {
                () if type_path.path.is_ident("bool") => BitFieldType::TypeBool,
                () if type_path.path.is_ident("u8") => BitFieldType::Type8,
                () if type_path.path.is_ident("u16") => BitFieldType::Type16,
                () if type_path.path.is_ident("u32") => BitFieldType::Type32,
                () if type_path.path.is_ident("u64") => BitFieldType::Type64,
                _ => BitFieldType::InvalidType { start: 0, end: 0 },
            },
            _ => BitFieldType::InvalidType { start: 0, end: 0 },
        }
    }
}

impl<'a> Into<BitFieldType> for &'a Type {
    fn into(self) -> BitFieldType {
        match self {
            Type::Path(type_path) => match () {
                () if type_path.path.is_ident("bool") => BitFieldType::TypeBool,
                () if type_path.path.is_ident("u8") => BitFieldType::Type8,
                () if type_path.path.is_ident("u16") => BitFieldType::Type16,
                () if type_path.path.is_ident("u32") => BitFieldType::Type32,
                () if type_path.path.is_ident("u64") => BitFieldType::Type64,
                _ => BitFieldType::InvalidType { start: 0, end: 0 },
            },
            _ => BitFieldType::InvalidType { start: 0, end: 0 },
        }
    }
}

impl BitField {
    /// The type required to fit the amount of bits desired.
    pub fn type_to_fit(&self, access: &Access, default_type: BitFieldType) -> BitFieldType {
        if matches!(access, Access::RWNS) {
            return default_type;
        }

        match self.bit_amount(default_type) {
            1 => BitFieldType::TypeBool,
            ..=8 => BitFieldType::Type8,
            ..=16 => BitFieldType::Type16,
            ..=32 => BitFieldType::Type32,
            ..=64 => BitFieldType::Type64,
            // FIXME
            _ => BitFieldType::InvalidType { start: 0, end: 0 },
        }
    }

    /// Get the offset of this bit from lsb.
    pub fn bit_offset(&self) -> usize {
        let Some((range_start, _)) = self.bits.into_range() else {
            return match self.bits {
                Bits::Single(ref single_bit) => single_bit.base10_parse().unwrap_or(0),
                _ => 0,
            };
        };

        match range_start {
            Bound::Included(v) | Bound::Excluded(v) => v,
            Bound::Unbounded => 0,
        }
    }

    /// Get the amount of bits required to read/write this field.
    pub fn bit_amount(&self, default_type: BitFieldType) -> usize {
        match self.bits.into_range() {
            None => 1,
            Some((range_start, range_end)) => {
                let start = match range_start {
                    Bound::Included(included) | Bound::Excluded(included) => included,
                    Bound::Unbounded => 0,
                };

                let end = match range_end {
                    Bound::Included(included) => included + 1,
                    Bound::Excluded(exclueded) => exclueded,
                    Bound::Unbounded => default_type.into(),
                };

                if start >= end {
                    0
                } else {
                    end - start
                }
            }
        }
    }

    /// Get the mask for the bit field
    pub fn bit_mask(&self, default_type: BitFieldType) -> u64 {
        self.bit_max(default_type) << self.bit_offset()
    }

    /// Maxium value allowed by this field
    ///
    /// *`bool`'s are just `1`*
    pub fn bit_max(&self, default_type: BitFieldType) -> u64 {
        2u64.pow(self.bit_amount(default_type) as u32) - 1
    }
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
    /// Read/Write No Shift
    RWNS,
}

mod access {
    syn::custom_keyword!(RW);
    syn::custom_keyword!(RO);
    syn::custom_keyword!(WO);
    syn::custom_keyword!(RW1C);
    syn::custom_keyword!(RW1O);
    syn::custom_keyword!(RWNS);
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
        } else if lookahead.peek(access::RWNS) {
            input.parse::<access::RWNS>()?;
            Ok(Access::RWNS)
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

impl Bits {
    pub fn into_range(&self) -> Option<(Bound<usize>, Bound<usize>)> {
        match self {
            Self::Range(expr) => {
                let start_number: Option<usize> =
                    expr.start.as_ref().and_then(|start| match start.as_ref() {
                        Expr::Lit(ExprLit {
                            attrs: _,
                            lit: Lit::Int(int),
                        }) => int.base10_parse().ok(),
                        _ => None,
                    });

                let end_number: Option<usize> =
                    expr.end.as_ref().and_then(|start| match start.as_ref() {
                        Expr::Lit(ExprLit {
                            attrs: _,
                            lit: Lit::Int(int),
                        }) => int.base10_parse().ok(),
                        _ => None,
                    });

                let start = match start_number {
                    Some(value) => Bound::Included(value),
                    None => Bound::Unbounded,
                };

                let end = match end_number {
                    Some(value) if matches!(expr.limits, syn::RangeLimits::HalfOpen(_)) => {
                        Bound::Excluded(value)
                    }
                    Some(value) => Bound::Included(value),
                    None => Bound::Unbounded,
                };

                Some((start, end))
            }
            _ => None,
        }
    }
}
