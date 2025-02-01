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

use std::collections::HashMap;

use proc_macro_error::emit_error;
use proc_macro2::Span;
use syn::{
    Attribute, Block, Expr, FnArg, Ident, ItemFn, LitBool, LitStr, ReturnType, Token, Visibility,
    parse::Parse, punctuated::Punctuated, spanned::Spanned,
};

#[derive(Debug)]
pub struct PortalMacroInput {
    pub args: PortalArgs,
    pub trait_input: PortalTrait,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum ProtocolKind {
    Unknown(Span),
    Syscall(Span),
    Ipc(Span),
}

impl Parse for ProtocolKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let string: LitStr = input.parse()?;
        let str_value = string.value();

        if &str_value == "syscall" {
            Ok(Self::Syscall(input.span()))
        } else if &str_value == "ipc" {
            Ok(Self::Ipc(input.span()))
        } else {
            emit_error!(
                string.span(),
                "Expected a protocol kind ('syscall', 'ipc'), found '{}'",
                str_value
            );
            Ok(Self::Unknown(input.span()))
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PortalArgs {
    pub global: Option<LitBool>,
    pub protocol: ProtocolKind,
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
#[allow(dead_code)]
pub struct PortalTrait {
    pub attr: Vec<Attribute>,
    pub vis: Visibility,
    pub portal_name: Ident,
    pub endpoints: Vec<PortalEndpoint>,
}

impl Parse for PortalTrait {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let _trait_token: Token![trait] = input.parse()?;
        let portal_name = input.parse()?;

        let inner;
        let _brace_token = syn::braced!(inner in input);
        let mut endpoints = Vec::new();

        loop {
            if inner.is_empty() {
                break;
            }

            while inner.peek(Token![;]) {
                inner.parse::<Token![;]>()?;
            }

            let item_fn: PortalEndpoint = match inner.parse() {
                Ok(v) => v,
                Err(err) => {
                    emit_error!(
                        err.span(),
                        "Cannot parse endpoint function: {}",
                        err.span().source_text().unwrap_or("??".into())
                    );

                    continue;
                }
            };

            endpoints.push(item_fn);
        }

        // Check if all endpoints have seperate IDs
        let mut ids_found: HashMap<usize, Span> = HashMap::new();

        for endpoint in &endpoints {
            match &endpoint.endpoint {
                EndpointKind::None => (),
                EndpointKind::Event(event_attribute) => {
                    if let Some(other_span) = ids_found.get(&event_attribute.id) {
                        let id = ids_found.keys().max().copied().unwrap_or(0) + 1;

                        emit_error!(
                            event_attribute.span,
                            "Cannot have two endpoint functions with the same ID ({})",
                            event_attribute.id;
                            help = "Try changing this ID to {}.", id;
                            node = other_span.span() => "Previous use of the ID {} here", event_attribute.id;
                        );
                    } else {
                        ids_found.insert(event_attribute.id, event_attribute.span);
                    }
                }
                EndpointKind::Handle(handle_attribute) => {
                    if let Some(other_span) = ids_found.get(&handle_attribute.id) {
                        let id = ids_found.keys().max().copied().unwrap_or(0) + 1;

                        emit_error!(
                            handle_attribute.span,
                            "Cannot have two endpoint functions with the same ID ({})",
                            handle_attribute.id;
                            help = "Try changing this ID to {}.", id;
                            node = other_span.span() => "Previous use of the ID {} here", handle_attribute.id;
                        );
                    } else {
                        ids_found.insert(handle_attribute.id, handle_attribute.span);
                    }
                }
            }
        }

        Ok(Self {
            attr,
            vis,
            portal_name,
            endpoints,
        })
    }
}

/*
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
    pub id: usize,
    pub span: Span,
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
    pub id: usize,
    pub span: Span,
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
    pub docs: Vec<Attribute>,
    pub endpoint: EndpointKind,
    pub fn_ident: Ident,
    pub input: Punctuated<FnArg, Token![,]>,
    pub output: ReturnType,
    pub is_unsafe: Option<Token![unsafe]>,
    pub fn_body: Box<Block>,
}

impl Parse for PortalEndpoint {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item_fn: ItemFn = input.parse()?;
        let fn_attributes = &item_fn.attrs;

        let mut docs = Vec::new();
        let mut endpoint = EndpointKind::None;

        for attr in fn_attributes {
            if attr.path().is_ident("doc") {
                docs.push(attr.clone());
            } else if attr.path().is_ident("event") {
                let name_value_attr_inner = attr.meta.require_name_value()?;
                match endpoint {
                    EndpointKind::Event(_) => emit_error!(
                        attr,
                        "Cannot define multiple #[event = ..] for a single event"
                    ),
                    EndpointKind::Handle(_) => emit_error!(
                        attr,
                        "A endpoint function can either be `event` or `handle` but never both"; help = "Remove either `#[event = ..]` or `#[handle = ..]`"
                    ),
                    EndpointKind::None => (),
                }
                match (&name_value_attr_inner.value).try_into() {
                    Ok(value) => endpoint = EndpointKind::Event(value),
                    Err(err) => {
                        emit_error!(attr, "Cannot parse #[event = ..] because {}", err)
                    }
                }
            } else if attr.path().is_ident("handle") {
                let name_value_attr_inner = attr.meta.require_name_value()?;
                match endpoint {
                    EndpointKind::Handle(_) => emit_error!(
                        attr,
                        "Cannot define multiple #[handle = ..] for a single handle"
                    ),
                    EndpointKind::Event(_) => emit_error!(
                        attr,
                        "A endpoint function can either be `event` or `handle` but never both"; help = "Remove either `#[event = ..]` or `#[handle = ..]`"
                    ),
                    EndpointKind::None => (),
                }
                match (&name_value_attr_inner.value).try_into() {
                    Ok(value) => endpoint = EndpointKind::Handle(value),
                    Err(err) => {
                        emit_error!(attr, "Cannot parse #[handle = ..] because {}", err)
                    }
                }
            } else {
                emit_error!(
                    attr,
                    "Unsupported attribute on portal: '{}'",
                    attr.span().source_text().unwrap_or("??".into())
                );
            }
        }

        if matches!(endpoint, EndpointKind::None) {
            emit_error!(
                item_fn,
                "This endpoint function must be either an event, or a handle.";
                help = "Consider adding either `#[event = 0]` or `#[handle = 0]`",
            );
        }

        item_fn.sig.inputs.iter().for_each(|a| match a {
            FnArg::Receiver(receiver) => {
                emit_error!(
                    receiver,
                    "Endpoints must not include `self` as an argument";
                    help = "Remove `self` from endpoint",
                );
            }
            _ => (),
        });

        let endpoint = Self {
            docs,
            endpoint,
            fn_ident: item_fn.sig.ident,
            input: item_fn.sig.inputs,
            output: item_fn.sig.output,
            is_unsafe: item_fn.sig.unsafety,
            fn_body: item_fn.block,
        };

        Ok(endpoint)
    }
}
