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

use proc_macro2::{Span, TokenStream};
use syn::Ident;

#[derive(Debug)]
pub enum PortalTypeKind {
    Unit,
    Bool,
    PtrValue {
        to_type: Box<PortalTypeKind>,
    },
    Str {
        allowed_values: Vec<String>,
        size_upper_limit: Option<usize>,
        size_lower_limit: Option<usize>,
    },
    UnsignedInt8 {
        lower_limit: Option<u8>,
        upper_limit: Option<u8>,
    },
    UnsignedInt16 {
        lower_limit: Option<u16>,
        upper_limit: Option<u16>,
    },
    UnsignedInt32 {
        lower_limit: Option<u32>,
        upper_limit: Option<u32>,
    },
    UnsignedInt64 {
        lower_limit: Option<u64>,
        upper_limit: Option<u64>,
    },
    UnsignedInt128 {
        lower_limit: Option<u128>,
        upper_limit: Option<u128>,
    },
    SignedInt8 {
        lower_limit: Option<i8>,
        upper_limit: Option<i8>,
    },
    SignedInt16 {
        lower_limit: Option<i16>,
        upper_limit: Option<i16>,
    },
    SignedInt32 {
        lower_limit: Option<i32>,
        upper_limit: Option<i32>,
    },
    SignedInt64 {
        lower_limit: Option<i64>,
        upper_limit: Option<i64>,
    },
    SignedInt128 {
        lower_limit: Option<i128>,
        upper_limit: Option<i128>,
    },
    Enum {
        type_name: String,
        inner_values: HashMap<String, PortalType>,
    },
    Struct {
        type_name: String,
        inner_values: HashMap<String, PortalType>,
    },
    UnsupportedType {
        ty: Box<syn::Type>,
    },
}

#[derive(Debug)]
pub struct PortalType {
    /// The name as its used in the input/output of a endpoint
    used_as_name: Option<String>,
    /// The kind and options of the value
    kind: PortalTypeKind,
    /// The span of the argument / value of this type
    span: Span,
}

pub fn generate_raw_input_enum(
    ident: Ident,
    inputs: HashMap<String, &[PortalType]>,
) -> TokenStream {
    todo!()
}

pub fn generate_raw_adapter_fn(inputs: PortalType, outputs: PortalType) -> TokenStream {
    todo!()
}
