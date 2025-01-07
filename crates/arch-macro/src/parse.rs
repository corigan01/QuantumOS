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

use std::ops::Bound;

use proc_macro_error::abort;
use syn::{Expr, ExprLit, ExprRange, ItemFn, Lit, LitInt, parse::Parse, spanned::Spanned};

#[derive(Debug)]
pub struct ParsedInterrupt {
    pub(crate) fn_tokens: ItemFn,
}

impl Parse for ParsedInterrupt {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            fn_tokens: input.parse()?,
        })
    }
}

#[derive(Debug)]
pub enum Bits {
    Single(LitInt),
    Range(ExprRange),
}

impl Parse for Bits {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr: Expr = match input.parse() {
            Ok(a) => a,
            Err(err) => proc_macro_error::abort!(
                err.span(),
                "Expected an index for IRQ number, or a range of IRQ numbers!";
                help = "Add an IRQ number to the attribute, like the following `#[interrupt(0)]`.";
            ),
        };
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
    fn span(&self) -> proc_macro2::Span {
        match self {
            Self::Single(s) => s.span(),
            Self::Range(r) => r.span(),
        }
    }
}

#[derive(Debug)]
pub struct ParsedInterruptArgs {
    bits: Bits,
}

impl ParsedInterruptArgs {
    pub fn span(&self) -> proc_macro2::Span {
        self.bits.span()
    }

    pub fn range(&self) -> (usize, usize) {
        match self.bits {
            Bits::Single(ref lit_int) => {
                let Ok(parsed) = lit_int.base10_parse() else {
                    abort!(self.bits.span(), "Must be a valid number between 0..255");
                };

                if parsed > 255 {
                    abort!(self.bits.span(), "Must be a valid number between 0..255");
                }

                (parsed, parsed)
            }
            Bits::Range(ref expr) => {
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
                    Some(value) => value,
                    None => 0,
                };

                let end = match end_number {
                    Some(value) if matches!(expr.limits, syn::RangeLimits::HalfOpen(_)) => {
                        value - 1
                    }
                    Some(value) => value,
                    None => 255,
                };

                if start > end {
                    abort!(expr.span(), "Start cannot be higher then end range");
                }

                if start > 255 || end > 255 {
                    abort!(expr.span(), "Must be a valid range from 0..255");
                }

                (start, end)
            }
        }
    }
}

impl Parse for ParsedInterruptArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            bits: input.parse()?,
        })
    }
}
