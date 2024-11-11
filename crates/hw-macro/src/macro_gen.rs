use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    make_hw_parse::{BitField, MakeHwMacroInput},
    provider_parse::MacroStruct,
};

pub struct GenMetadata {
    /// Is this a field, or is it a constant function?
    const_possible: bool,
    /// Is this a function call?
    is_function: bool,
    /// Reading/Writing from/to this function/field requires the need for
    ///
    /// - `None` => No self, independent function
    /// - `Some(false)` => `&self`
    /// - `Some(true)` => `&mut self`
    need_mut: Option<bool>,
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

struct Fields<'a> {
    fields: &'a MakeHwMacroInput,
    read_gen: Option<&'a dyn GenRead>,
    write_gen: Option<&'a dyn GenWrite>,
}

impl<'a> Fields<'a> {
    pub fn new(
        fields: &'a MakeHwMacroInput,
        read_gen: Option<&'a dyn GenRead>,
        write_gen: Option<&'a dyn GenWrite>,
    ) -> Self {
        Self {
            fields,
            read_gen,
            write_gen,
        }
    }

    pub fn generate_field(&self, field: &BitField) -> TokenStream {
        todo!()
    }

    pub fn generate_fields(&self) -> TokenStream {
        let fields: Vec<_> = self
            .fields
            .fields
            .iter()
            .map(|field| self.generate_field(field))
            .collect();

        quote! {
            #(#fields)*
        }
    }
}

pub fn gen_struct(macro_struct: MacroStruct) -> TokenStream {
    let fields = Fields::new(&macro_struct.macro_fields, None, None);

    fields.generate_fields();
    quote! {}
}
