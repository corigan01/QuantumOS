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

use proc_macro_error::{abort, emit_error};
use proc_macro2::Span;
use syn::{
    Attribute, Expr, FnArg, Ident, ItemFn, LitBool, LitStr, ReturnType, Signature, Token,
    Visibility, parse::Parse, punctuated::Punctuated, spanned::Spanned, token,
};

#[derive(Clone, Copy, Debug)]
pub enum ProtocolKind {
    Syscall(Span),
}

impl Parse for ProtocolKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let string: LitStr = input.parse()?;
        let str_value = string.value();

        if &str_value == "syscall" {
            Ok(Self::Syscall(input.span()))
        } else {
            abort!(string.span(), "Expected a protocol (ie.'syscall', ...)")
        }
    }
}

#[derive(Debug)]
pub struct PortalArgs {
    global: Option<LitBool>,
    protocol: ProtocolKind,
}

mod portal_keywords {
    // Portal Args
    syn::custom_keyword!(global);
    syn::custom_keyword!(protocol);
}

impl Parse for PortalArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut global = None;
        let mut protocol = None;

        loop {
            if input.is_empty() {
                break;
            }

            let lookahead = input.lookahead1();
            if lookahead.peek(portal_keywords::global) {
                input.parse::<portal_keywords::global>()?;
                input.parse::<Token![=]>()?;
                global = Some(input.parse()?);
            } else if lookahead.peek(portal_keywords::protocol) {
                input.parse::<portal_keywords::protocol>()?;
                input.parse::<Token![=]>()?;
                protocol = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            global,
            protocol: protocol.unwrap_or(ProtocolKind::Syscall(Span::call_site())),
        })
    }
}

#[derive(Debug)]
pub struct PortalTrait {
    attr: Vec<Attribute>,
    vis: Visibility,
    trait_token: Token![trait],
    portal_name: Ident,
    brace_token: token::Brace,
    endpoints: Punctuated<PortalEndpoint, Token![;]>,
}

impl Parse for PortalTrait {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inner;
        Ok(Self {
            attr: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            trait_token: input.parse()?,
            portal_name: input.parse()?,
            brace_token: syn::braced!(inner in input),
            endpoints: inner.parse_terminated(PortalEndpoint::parse, Token![;])?,
        })
    }
}

/*
  #[stable(since = "0.1.0")]
  #[event(id = 0)]
  fn exit(exit_reson: ExitReason) -> ! {
    enum ExitReason {
      Success,
      Failure
    }
  }
*/

#[derive(Debug)]
pub struct EventAttribute {
    id: usize,
    span: Span,
}

impl TryFrom<&Expr> for EventAttribute {
    type Error = syn::Error;

    fn try_from(value: &Expr) -> Result<Self, Self::Error> {
        match value {
            Expr::Lit(expr_lit) => match &expr_lit.lit {
                syn::Lit::Int(lit_int) => Ok(Self {
                    id: lit_int.base10_parse()?,
                    span: expr_lit.span(),
                }),
                _ => Err(syn::Error::new(
                    expr_lit.span(),
                    format!(
                        "expected integer literal, found '{}'",
                        expr_lit.span().source_text().unwrap_or("??".into())
                    ),
                )),
            },
            _ => Err(syn::Error::new(
                value.span(),
                format!(
                    "expected literal, found '{}'",
                    value.span().source_text().unwrap_or("??".into())
                ),
            )),
        }
    }
}

#[derive(Debug)]
pub struct HandleAttribute {
    id: usize,
    span: Span,
}

impl TryFrom<&Expr> for HandleAttribute {
    type Error = syn::Error;

    fn try_from(value: &Expr) -> Result<Self, Self::Error> {
        match value {
            Expr::Lit(expr_lit) => match &expr_lit.lit {
                syn::Lit::Int(lit_int) => Ok(Self {
                    id: lit_int.base10_parse()?,
                    span: expr_lit.span(),
                }),
                _ => Err(syn::Error::new(
                    expr_lit.span(),
                    format!(
                        "expected integer literal, found '{}'",
                        expr_lit.span().source_text().unwrap_or("??".into())
                    ),
                )),
            },
            _ => Err(syn::Error::new(
                value.span(),
                format!(
                    "expected literal, found '{}'",
                    value.span().source_text().unwrap_or("??".into())
                ),
            )),
        }
    }
}

#[derive(Debug)]
pub enum EndpointKind {
    None,
    Event(EventAttribute),
    Handle(HandleAttribute),
}

#[derive(Debug)]
pub struct PortalEndpoint {
    docs: Vec<Attribute>,
    endpoint: EndpointKind,
    fn_ident: Ident,
    input: Punctuated<FnArg, Token![,]>,
    output: ReturnType,
    is_unsafe: Option<Token![unsafe]>,
}

impl Parse for PortalEndpoint {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item_fn: ItemFn = input.parse()?;
        let fn_attributes = item_fn.attrs;

        let mut docs = Vec::new();
        let mut endpoint = EndpointKind::None;

        for attr in fn_attributes {
            let name_value_attr_inner = attr.meta.require_name_value()?;
            if name_value_attr_inner.path.is_ident("doc") {
                docs.push(attr);
            } else if name_value_attr_inner.path.is_ident("event") {
                if matches!(endpoint, EndpointKind::Event(_)) {
                    emit_error!(
                        attr.span(),
                        "Cannot define multiple #[event = ..] for a single event"
                    );
                }
                if matches!(endpoint, EndpointKind::Handle(_)) {
                    emit_error!(
                        attr.span(),
                        "A endpoint function can either be `event` or `handle` but never both"; help = "Remove either `#[event = ..]` or `#[handle = ..]`"
                    );
                }
                match (&name_value_attr_inner.value).try_into() {
                    Ok(value) => endpoint = EndpointKind::Event(value),
                    Err(err) => {
                        emit_error!(attr.span(), "Cannot parse #[event = ..] because {}", err)
                    }
                }
            } else if name_value_attr_inner.path.is_ident("handle") {
                if matches!(endpoint, EndpointKind::Handle(_)) {
                    emit_error!(
                        attr.span(),
                        "Cannot define multiple #[handle = ..] for a single handle"
                    );
                }
                if matches!(endpoint, EndpointKind::Event(_)) {
                    emit_error!(
                        attr.span(),
                        "A endpoint function can either be `event` or `handle` but never both"; help = "Remove either `#[event = ..]` or `#[handle = ..]`"
                    );
                }
                match (&name_value_attr_inner.value).try_into() {
                    Ok(value) => endpoint = EndpointKind::Handle(value),
                    Err(err) => {
                        emit_error!(attr.span(), "Cannot parse #[handle = ..] because {}", err)
                    }
                }
            }
        }

        Ok(Self {
            docs,
            endpoint,
            fn_ident: item_fn.sig.ident,
            input: item_fn.sig.inputs,
            output: item_fn.sig.output,
            is_unsafe: item_fn.sig.unsafety,
        })
    }
}
