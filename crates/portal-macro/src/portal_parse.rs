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

use proc_macro2::Span;
use syn::{LitBool, Token, parse::Parse};

#[derive(Clone, Copy, Debug)]
pub enum ProtocolKind {
    Syscall(Span),
}

impl Parse for ProtocolKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(portal_keywords::syscall) {
            let kw: portal_keywords::syscall = input.parse()?;
            Ok(Self::Syscall(kw.span))
        } else {
            Err(lookahead.error())
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

    // Protocol Kind
    syn::custom_keyword!(syscall);
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
        }

        Ok(Self {
            global,
            protocol: protocol.unwrap_or(ProtocolKind::Syscall(Span::call_site())),
        })
    }
}
