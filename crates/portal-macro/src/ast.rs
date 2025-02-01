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
use syn::{Attribute, FnArg, Ident, ItemEnum, ReturnType, Visibility};

#[derive(Debug)]
pub struct PortalMacro {
    pub doc_attributes: Vec<Attribute>,
    pub args: Option<PortalMacroArgs>,
    pub vis: Visibility,
    pub trait_ident: Ident,
    pub endpoints: Vec<ProtocolEndpoint>,
}

#[derive(Debug)]
pub struct PortalMacroArgs {
    pub protocol_kind: ProtocolKind,
    pub is_global: bool,
}

#[derive(Debug)]
pub enum ProtocolKind {
    Syscall,
    Ipc,
    Invalid,
}

#[derive(Debug)]
pub struct ProtocolEndpoint {
    pub doc_attributes: Vec<Attribute>,
    pub portal_id: (usize, Span),
    pub kind: ProtocolEndpointKind,
    pub fn_ident: Ident,
    pub input_args: Vec<ProtocolInputArg>,
    pub output_arg: ProtocolOutputArg,
    pub is_unsafe: bool,
    pub body: Vec<ProtocolDefine>,
}

#[derive(Debug)]
pub enum ProtocolVarType {
    ResultKind(Box<ProtocolVarType>, Box<ProtocolVarType>),
    Never,
    Unit,
    Signed8,
    Signed16,
    Signed32,
    Signed64,
    Unsigned8,
    Unsigned16,
    Unsigned32,
    Unsigned64,
    UserDefined(Ident),
    Str,
    RefTo {
        is_mut: bool,
        to: Box<ProtocolVarType>,
    },
    PtrTo {
        is_mut: bool,
        to: Box<ProtocolVarType>,
    },
}

#[derive(Debug)]
pub struct ProtocolInputArg {
    pub argument_ident: Ident,
    pub ty: ProtocolVarType,
}

#[derive(Debug)]
pub struct ProtocolOutputArg(pub ProtocolVarType);

#[derive(Debug)]
pub enum ProtocolEndpointKind {
    Event,
}

#[derive(Debug)]
pub enum ProtocolDefine {
    DefinedEnum(Box<ItemEnum>),
}

impl PortalMacro {
    /// Get all the not unique portal ids
    pub fn all_non_unique_portal_ids(&self) -> impl Iterator<Item = (usize, Span, Span)> {
        // FIXME: Maybe there is a less slow way of doing this?
        self.endpoints
            .iter()
            .enumerate()
            .flat_map(|(our_index, endpoint)| {
                let (our_id, our_span) = endpoint.portal_id;

                self.endpoints.iter().enumerate().skip(our_index).find_map(
                    |(_, other_endpoints)| {
                        let (other_id, other_span) = other_endpoints.portal_id;

                        if other_id == our_id {
                            Some((other_id, our_span, other_span))
                        } else {
                            None
                        }
                    },
                )
            })
    }

    /// Get the highest protocol endpoint ID
    pub fn highest_id(&self) -> usize {
        self.endpoints
            .iter()
            .map(|endpoint| endpoint.portal_id.0)
            .max()
            .unwrap_or(0)
    }
}
