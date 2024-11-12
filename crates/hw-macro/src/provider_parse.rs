use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, ItemStruct, Type};

use crate::make_hw_parse::{BitFieldType, MakeHwMacroInput};

#[derive(Debug)]
pub struct MacroStruct {
    pub(crate) struct_inner: ItemStruct,
    pub(crate) macro_fields: MakeHwMacroInput,
}

pub struct GenMetadata {
    /// Is this a field, or is it a constant function?
    pub const_possible: bool,
    /// Is this a function call?
    pub is_function: bool,
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

impl GenRead for MacroStruct {
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
                is_function: false,
                need_mut: Some(false),
                data_type: ty.into(),
                carry_self: true,
                is_unsafe: false,
            }
        } else {
            todo!()
        }
    }
}

impl GenWrite for MacroStruct {
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
                is_function: false,
                need_mut: Some(true),
                data_type: ty.into(),
                carry_self: true,
                is_unsafe: false,
            }
        } else {
            todo!()
        }
    }
}

impl MacroStruct {
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
