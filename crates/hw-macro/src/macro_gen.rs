use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;

use crate::{
    make_hw_parse::{Access, BitField, BitFieldType, Bits, MakeHwMacroInput},
    provider_parse::MacroStruct,
};

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

    /// Generate the inner function contents for a write
    fn gen_inner_write(&self, field: &BitField) -> TokenStream {
        let (Some(write_gen), Some(read_gen)) = (self.write_gen, self.read_gen) else {
            field
                .keyword
                .span()
                .unwrap()
                .error("No read/write function found!")
                .span_help(
                    field.paren_token.span.span().unwrap(),
                    "Define a 'write' and a 'read' function, or define a field to read/write into.",
                )
                .emit();

            return quote! {};
        };

        let write_metadata = write_gen.metadata();
        let read_metadata = read_gen.metadata();

        let write_means = write_gen.gen_write();
        let read_means = read_gen.gen_read();

        let smallest_type = write_metadata.data_type.min(read_metadata.data_type);

        println!(
            "Read = {:?}, Write = {:?}, Smallest = {:?}",
            read_metadata.data_type, write_metadata.data_type, smallest_type
        );

        let mut token_vector: Vec<TokenStream> = Vec::new();

        token_vector.push(quote! {});

        let offset_bit = field.bit_offset();
        let bit_amount = field.bit_amount(smallest_type);

        match field.type_to_fit(smallest_type) {
            BitFieldType::InvalidType { start, end } => {
                field
                    .keyword
                    .span()
                    .unwrap()
                    .error(format!("Invalid Bit Argument, Start={} End={}", start, end))
                    .emit();
            }
            BitFieldType::TypeBool => {
                token_vector.push(quote! {
                    #write_means = if flag {
                        read_value | (1 << offset_bit);
                    } else {
                        read_value & !(1 << offset_bit);
                    };
                });
            }
            // If no casting is required
            multi_bit_type if write_metadata.data_type == read_metadata.data_type => {
                token_vector.push(quote! {});
            }
            _ => (),
        }

        quote! {
            #(#token_vector)*
        }
    }

    pub fn generate_field(&self, field: &BitField) -> TokenStream {
        let (read_header, read_footer, write_header, write_footer) =
            match (&field.access, &field.bits) {
                (Access::RW1C, Bits::Single(_)) => ("is_", "_clear", "clear_", "_flag"),
                (Access::RW1O, Bits::Single(_)) => ("is_", "_active", "activate_", "_flag"),

                (Access::RW1C, Bits::Range(_)) => ("read_", "_clear", "clear_", ""),
                (Access::RW1O, Bits::Range(_)) => ("read_", "_active", "activate_", ""),

                (Access::RW, Bits::Single(_)) => ("is_", "_set", "set_", "_flag"),
                (Access::RW, Bits::Range(_)) => ("set_", "", "get_", ""),

                (Access::RO, Bits::Single(_)) => ("is_", "_set", "", ""),
                (Access::RO, Bits::Range(_)) => ("read_", "", "", ""),

                (Access::WO, Bits::Single(_)) => ("", "", "set_", "_flag"),
                (Access::WO, Bits::Range(_)) => ("", "", "read_", ""),
            };

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
