use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, Visibility};

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

pub struct GenInfo {
    pub gen_const: bool,
    pub gen_unsafe: bool,
    pub function_self_mut: Option<bool>,
    pub function_type: TokenStream,
    pub bit_offset: usize,
    pub bit_amount: usize,
    pub bit_mask: u64,
    pub vis: Visibility,
    pub function_ident: String,
    pub carry_self: bool,
    pub span: Span,
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
    fn gen_write(&self, gen_info: GenInfo) -> TokenStream {
        let (Some(read_gen), Some(write_gen)) = (self.read_gen, self.write_gen) else {
            return quote! {};
        };

        let mut tokens: Vec<TokenStream> = Vec::new();

        let vis = gen_info.vis;
        tokens.push(quote! {#vis});

        if gen_info.gen_const {
            tokens.push(quote! { const });
        }

        if gen_info.gen_unsafe {
            tokens.push(quote! { unsafe });
        }

        let ident = gen_info.function_ident;
        tokens.push(quote! { fn #ident });
        let input_type = gen_info.function_type;

        tokens.push(match gen_info.function_self_mut {
            Some(true) => quote! { (&mut self, value: #input_type )},
            Some(false) => quote! { (&self, value: #input_type )},
            None => quote! { (value: #input_type )},
        });

        let post_gen_tokens = if gen_info.carry_self {
            tokens.push(quote! { -> Self });

            quote! {*self}
        } else {
            quote! {}
        };

        // This will make the 'read_value' variable
        let read_value = read_gen.gen_read();
        // This will take the 'write_value' variable
        let write_value = write_gen.gen_write();

        let bit_offset = gen_info.bit_offset;
        let bit_mask = gen_info.bit_mask;

        if gen_info.bit_amount == 1 {
            tokens.push(quote! {{
                // Read
                #read_value;

                // Modify
                let write_value = if value {
                    read_value | (1 << #bit_offset)
                } else {
                    read_value & !(1 << #bit_offset)
                };

                // Write
                #write_value;

                // Post Gen
                #post_gen_tokens
            }});
        } else {
            tokens.push(quote! {{
                // Read
                #read_value;

                // Modify
                let write_value =
                    (read_value & !#bit_mask) | (value << #bit_offset);

                // Write
                #write_value;

                // Post Gen
                #post_gen_tokens
            }});
        }

        quote! {
            #(#tokens)*
        }
    }

    pub fn gen_read(&self, gen_info: GenInfo) -> TokenStream {
        let Some(read_gen) = self.read_gen else {
            return quote! {};
        };

        let mut tokens: Vec<TokenStream> = Vec::new();

        let vis = gen_info.vis;
        tokens.push(quote! {#vis});

        if gen_info.gen_const {
            tokens.push(quote! { const });
        }

        if gen_info.gen_unsafe {
            tokens.push(quote! { unsafe });
        }

        let ident = gen_info.function_ident;
        tokens.push(quote! { fn #ident });
        let output_type = gen_info.function_type;

        tokens.push(match gen_info.function_self_mut {
            Some(true) => quote! { (&mut self)},
            Some(false) => quote! { (&self)},
            None => quote! { () },
        });

        if gen_info.carry_self {
            tokens.push(quote! { -> #output_type });
        }

        // This will make the 'read_value' variable
        let read_value = read_gen.gen_read();

        let bit_offset = gen_info.bit_offset;
        let bit_mask = gen_info.bit_mask;

        if gen_info.bit_amount == 1 {
            tokens.push(quote! {{
                // Read
                #read_value;

                // Pull out value
                read_value & (1 << #bit_offset) != 0

            }});
        } else {
            tokens.push(quote! {{
                // Read
                #read_value;

                // Pull out value
                (read_value & #bit_mask) >> #bit_offset
            }});
        }

        quote! {
            #(#tokens)*
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

        let function_type = todo!();
        let bit_offset = todo!();
        let bit_amount = todo!();
        let bit_mask = todo!();

        let tokens: Vec<TokenStream> = Vec::new();

        // Read
        if !matches!(field.access, Access::WO) {
            let Some(read_gen) = self.read_gen else {
                field
                    .keyword
                    .span()
                    .unwrap()
                    .error("Could not find a valid read functions")
                    .help("Define a field/function to read to this bit field.")
                    .emit();

                return quote! {};
            };

            let read_ident = format!("{read_header}{}{read_footer}", field.ident);
            let read_meta = read_gen.metadata();

            let gen_info_read = GenInfo {
                gen_const: read_meta.const_possible,
                gen_unsafe: read_meta.is_unsafe,
                function_self_mut: read_meta.need_mut,
                function_type,
                bit_offset,
                bit_amount,
                bit_mask,
                vis: field.vis,
                function_ident: read_ident,
                carry_self: false,
                span: field.keyword.span(),
            };

            tokens.push(self.gen_read(gen_info_read));
        }

        if !matches!(field.access, Access::RO) {
            let (Some(read_gen), Some(write_gen)) = (self.read_gen, self.write_gen) else {
                field
                    .keyword
                    .span()
                    .unwrap()
                    .error("Could not find a valid read/write functions")
                    .help("Define a field/function to read/write to this bit field.")
                    .emit();

                return quote! {};
            };

            let write_ident = format!("{write_header}{}{write_footer}", field.ident);
            let read_meta = read_gen.metadata();
            let write_meta = write_gen.metadata();

            let gen_info_write = GenInfo {
                gen_const: read_meta.const_possible && write_meta.const_possible,
                gen_unsafe: read_meta.is_unsafe || write_meta.is_unsafe,
                function_self_mut: read_meta.need_mut.or(write_meta.need_mut).map(|_| {
                    read_meta.need_mut.unwrap_or(false) || write_meta.need_mut.unwrap_or(false)
                }),
                function_type,
                bit_offset,
                bit_amount,
                bit_mask,
                vis: field.vis,
                function_ident: write_ident,
                carry_self: write_meta.carry_self,
                span: field.keyword.span(),
            };

            tokens.push(self.gen_write(gen_info_write));
        }

        quote! {
            #(#tokens)*
        }
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
