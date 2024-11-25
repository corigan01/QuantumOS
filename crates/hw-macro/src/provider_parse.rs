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

use core::panic;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    visit::{self, Visit},
    Field, ItemFn, ItemMod, ItemStruct, Type,
};

use crate::make_hw_parse::{BitFieldType, MakeHwMacroInput};

#[derive(Debug)]
pub enum MacroImplWho<'ast> {
    Nothing,
    IsStruct(&'ast ItemStruct),
    IsMod(&'ast ItemMod),
}

impl<'ast> Visit<'ast> for MacroImplWho<'ast> {
    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        *self = Self::IsMod(i);
        visit::visit_item_mod(self, i);
    }

    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        *self = Self::IsStruct(i);
        visit::visit_item_struct(self, i);
    }
}

#[derive(Debug)]
pub struct MacroStruct<'a> {
    pub(crate) struct_inner: &'a ItemStruct,
    pub(crate) macro_fields: MakeHwMacroInput,
}

#[derive(Debug)]
pub struct MacroMod<'a> {
    pub(crate) mod_inner: &'a ItemMod,
    pub(crate) macro_fields: MakeHwMacroInput,
}

pub struct GenMetadata {
    /// Is this a field, or is it a constant function?
    pub const_possible: bool,
    /// Reading/Writing from/to this function/field requires the need for
    ///
    /// - `None` => No self, independent function
    /// - `Some(false)` => `&self`
    /// - `Some(true)` => `&mut self`
    pub need_mut: Option<bool>,
    /// Data type of input/output.
    pub data_type: BitFieldType,
    /// An option if we can carry ourself (aka. clone and return self on write).
    pub carry_self: bool,
    /// Does this function need to be unsafe?
    pub is_unsafe: bool,
}

pub trait GenRead {
    /// Generate the means to read from this field/function.
    fn gen_read(&self) -> TokenStream;
    /// Gather metadata about this generator.
    fn metadata(&self) -> GenMetadata;
}

pub trait GenWrite {
    /// Generate the means to write to this field/function.
    fn gen_write(&self) -> TokenStream;
    /// Gather metadata about this generator.
    fn metadata(&self) -> GenMetadata;
}

impl<'a> GenRead for MacroStruct<'a> {
    fn gen_read(&self) -> TokenStream {
        if let Some(ty) = self.is_single_field_struct() {
            quote! { let read_value: #ty = self.0; }
        } else {
            todo!()
        }
    }

    fn metadata(&self) -> GenMetadata {
        if let Some(ty) = self.is_single_field_struct() {
            GenMetadata {
                const_possible: true,
                need_mut: Some(false),
                data_type: ty.into(),
                carry_self: false,
                is_unsafe: false,
            }
        } else {
            todo!()
        }
    }
}

impl<'a> GenWrite for MacroStruct<'a> {
    fn gen_write(&self) -> TokenStream {
        if let Some(ty) = self.is_single_field_struct() {
            quote! { self.0 = (write_value) as #ty; }
        } else {
            todo!()
        }
    }

    fn metadata(&self) -> GenMetadata {
        if let Some(ty) = self.is_single_field_struct() {
            GenMetadata {
                const_possible: true,
                need_mut: Some(true),
                data_type: ty.into(),
                carry_self: self.is_copy(),
                is_unsafe: false,
            }
        } else {
            todo!()
        }
    }
}

impl<'b> MacroStruct<'b> {
    pub fn is_copy(&self) -> bool {
        // FIXME: We cannot really figure out if the type impl Clone
        //        other than looking at the derive macro. There could
        //        be another way for the user to tell us if the type
        //        can be carried or not.
        self.struct_inner.attrs.iter().any(|attribute| {
            let mut impl_clone = false;

            if attribute.meta.path().is_ident("derive") {
                attribute
                    .parse_nested_meta(|meta| {
                        if meta.path.is_ident("Clone") {
                            impl_clone = true;
                        }

                        Ok(())
                    })
                    .unwrap();
            }

            impl_clone
        })
    }

    pub fn is_single_field_struct(&self) -> Option<Type> {
        match self.struct_inner.fields {
            syn::Fields::Unnamed(ref fields_unnamed) => {
                let fields: Vec<&Field> = fields_unnamed.unnamed.iter().collect();

                if fields.len() == 1 {
                    Some(fields[0].ty.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn gen_reading<'a>(&'a self) -> Option<&'a dyn GenRead> {
        if self.is_single_field_struct().is_some() {
            Some(self)
        } else {
            None
        }
    }

    pub fn gen_writing<'a>(&'a self) -> Option<&'a dyn GenWrite> {
        if self.is_single_field_struct().is_some() {
            Some(self)
        } else {
            None
        }
    }
}

impl<'a> GenRead for MacroMod<'a> {
    fn gen_read(&self) -> TokenStream {
        let meta = GenRead::metadata(self);
        let ty: TokenStream = meta.data_type.into();

        quote! { let read_value: #ty = read(); }
    }

    fn metadata(&self) -> GenMetadata {
        let Some(read_fn) = self.get_read_function() else {
            // TODO: Don't panic, make errors
            panic!("No 'read' function defined!");
        };

        let sig = &read_fn.sig;
        let return_type = match &sig.output {
            syn::ReturnType::Default => {
                panic!("'read' function does not have a valid return type!");
            }
            syn::ReturnType::Type(_, kind) => kind.as_ref().into(),
        };

        GenMetadata {
            const_possible: sig.constness.is_some(),
            need_mut: None,
            data_type: return_type,
            carry_self: false,
            is_unsafe: sig.unsafety.is_some(),
        }
    }
}

impl<'a> GenWrite for MacroMod<'a> {
    fn gen_write(&self) -> TokenStream {
        let meta = GenWrite::metadata(self);
        let ty: TokenStream = meta.data_type.into();

        quote! { write(write_value as #ty); }
    }

    fn metadata(&self) -> GenMetadata {
        let Some(write_fn) = self.get_write_function() else {
            // TODO: Don't panic, make errors
            panic!("No 'write' function defined!");
        };

        let sig = &write_fn.sig;
        let input_type = match sig.inputs.first() {
            Some(syn::FnArg::Typed(arg)) => arg.ty.as_ref().into(),
            _ => panic!("'write' function does not have a valid input type!"),
        };

        GenMetadata {
            const_possible: sig.constness.is_some(),
            need_mut: None,
            data_type: input_type,
            carry_self: false,
            is_unsafe: sig.unsafety.is_some(),
        }
    }
}

#[derive(Debug)]
struct RwFunctionVisitor<'ast> {
    read_fn: Option<&'ast ItemFn>,
    write_fn: Option<&'ast ItemFn>,
}

impl<'ast> RwFunctionVisitor<'ast> {
    pub fn new() -> Self {
        Self {
            read_fn: None,
            write_fn: None,
        }
    }
}

impl<'ast> Visit<'ast> for RwFunctionVisitor<'ast> {
    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        if i.sig.ident == "read" {
            self.read_fn = Some(i);
        } else if i.sig.ident == "write" {
            self.write_fn = Some(i);
        }

        syn::visit::visit_item_fn(self, i);
    }
}

impl<'b> MacroMod<'b> {
    pub fn get_read_function(&self) -> Option<&'b ItemFn> {
        // FIXME: We should figure out how to cache this at some point
        let mut fn_visitor = RwFunctionVisitor::new();
        fn_visitor.visit_item_mod(&self.mod_inner);

        fn_visitor.read_fn
    }

    pub fn get_write_function(&self) -> Option<&'b ItemFn> {
        // FIXME: We should figure out how to cache this at some point
        let mut fn_visitor = RwFunctionVisitor::new();
        fn_visitor.visit_item_mod(&self.mod_inner);

        fn_visitor.write_fn
    }

    pub fn gen_reading<'a>(&'a self) -> Option<&'a dyn GenRead> {
        if self.get_read_function().is_some() {
            Some(self)
        } else {
            None
        }
    }

    pub fn gen_writing<'a>(&'a self) -> Option<&'a dyn GenWrite> {
        if self.get_write_function().is_some() {
            Some(self)
        } else {
            None
        }
    }
}
