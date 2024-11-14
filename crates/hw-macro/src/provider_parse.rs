use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::Parse,
    visit::{self, Visit},
    Field, ItemMod, ItemStruct, Type,
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
        todo!()
    }

    fn metadata(&self) -> GenMetadata {
        todo!()
    }
}

impl<'a> GenWrite for MacroMod<'a> {
    fn gen_write(&self) -> TokenStream {
        todo!()
    }

    fn metadata(&self) -> GenMetadata {
        todo!()
    }
}

impl<'b> MacroMod<'b> {
    pub fn gen_reading<'a>(&'a self) -> Option<&'a dyn GenRead> {
        Some(self)
    }

    pub fn gen_writing<'a>(&'a self) -> Option<&'a dyn GenWrite> {
        Some(self)
    }
}
