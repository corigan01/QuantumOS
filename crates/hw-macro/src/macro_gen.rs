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

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, Attribute, Ident, Visibility};

use crate::{
    make_hw_parse::{Access, BitField, BitFieldType, Bits, MakeHwMacroInput},
    provider_parse::{GenRead, GenWrite, MacroMod, MacroStruct},
};

pub struct GenInfo<'a> {
    pub gen_const: bool,
    pub gen_unsafe: bool,
    pub function_self_mut: Option<bool>,
    pub function_type: TokenStream,
    pub inner_type: TokenStream,
    pub bit_offset: usize,
    pub bit_amount: usize,
    pub bit_mask: u64,
    pub vis: Visibility,
    pub function_ident: Ident,
    pub carry_self: bool,
    pub attributes: &'a [Attribute],
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

        let attributes = gen_info.attributes;
        tokens.push(quote! {
            #(#attributes)*
        });

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
        let inner_type = gen_info.inner_type;

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
                    (read_value & !(#bit_mask as #inner_type)) | ((value as #inner_type) << (#bit_offset as #inner_type));

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

        let attributes = gen_info.attributes;
        tokens.push(quote! {
            #(#attributes)*
        });

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

        tokens.push(quote! { -> #output_type });

        // This will make the 'read_value' variable
        let read_value = read_gen.gen_read();

        let bit_offset = gen_info.bit_offset;
        let bit_mask = gen_info.bit_mask;
        let inner_type = gen_info.inner_type;

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
                ((read_value & (#bit_mask as #inner_type)) >> (#bit_offset as #inner_type)) as #output_type
            }});
        }

        quote! {
            #(#tokens)*
        }
    }

    fn default_type(&self) -> BitFieldType {
        let write = self
            .write_gen
            .map(|write_gen| write_gen.metadata().data_type);
        let read = self.read_gen.map(|read_gen| read_gen.metadata().data_type);

        if !write.is_some_and(|write_type| read.is_some_and(|read_type| write_type == read_type)) {
            // TODO: Make the span better on this
            Span::call_site()
                .unwrap()
                .error("'read' and 'write' function types do not match!")
                .emit();
        }

        // TODO: We should figure out a better way to do this
        write.unwrap_or(read.unwrap_or(BitFieldType::InvalidType { start: 0, end: 0 }))
    }

    pub fn generate_field(&self, field: &BitField) -> TokenStream {
        let (read_header, read_footer, write_header, write_footer) =
            match (&field.access, &field.bits) {
                (Access::RW1C, Bits::Single(_)) => ("is_", "_clear", "clear_", "_flag"),
                (Access::RW1O, Bits::Single(_)) => ("is_", "_active", "activate_", "_flag"),

                (Access::RW1C, Bits::Range(_)) => ("read_", "_clear", "clear_", ""),
                (Access::RW1O, Bits::Range(_)) => ("read_", "_active", "activate_", ""),

                (Access::RW, Bits::Single(_)) => ("is_", "_set", "set_", "_flag"),
                (Access::RW, Bits::Range(_)) => ("get_", "", "set_", ""),

                (Access::RO, Bits::Single(_)) => ("is_", "_set", "", ""),
                (Access::RO, Bits::Range(_)) => ("read_", "", "", ""),

                (Access::WO, Bits::Single(_)) => ("", "", "set_", "_flag"),
                (Access::WO, Bits::Range(_)) => ("", "", "set_", ""),
            };

        let default_type = self.default_type();
        let function_type: TokenStream = field.type_to_fit(default_type).into();

        let bit_offset = field.bit_offset();
        let bit_amount = field.bit_amount(default_type);
        let bit_mask = field.bit_mask(default_type);

        let mut tokens: Vec<TokenStream> = Vec::new();

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

            let read_ident = Ident::new(
                &format!("{read_header}{}{read_footer}", field.ident),
                Span::call_site(),
            );
            let read_meta = read_gen.metadata();

            let gen_info_read = GenInfo {
                gen_const: read_meta.const_possible,
                gen_unsafe: read_meta.is_unsafe,
                function_self_mut: read_meta.need_mut,
                function_type: function_type.clone(),
                bit_offset,
                bit_amount,
                bit_mask,
                vis: field.vis.clone(),
                function_ident: read_ident,
                carry_self: false,
                inner_type: default_type.into(),
                attributes: &field.attr,
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

            let write_ident = Ident::new(
                &format!("{write_header}{}{write_footer}", field.ident),
                Span::call_site(),
            );
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
                vis: field.vis.clone(),
                function_ident: write_ident,
                carry_self: write_meta.carry_self,
                inner_type: default_type.into(),
                attributes: &field.attr,
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
    let fields = Fields::new(
        &macro_struct.macro_fields,
        macro_struct.gen_reading(),
        macro_struct.gen_writing(),
    );

    let struct_gen = &macro_struct.struct_inner;

    let struct_ident = &macro_struct.struct_inner.ident;
    let fields = fields.generate_fields();

    quote! {
        #struct_gen

        #[automatically_derived]
        impl #struct_ident {
            #fields
        }
    }
}

pub fn gen_mod(macro_mod: MacroMod) -> TokenStream {
    let fields = Fields::new(
        &macro_mod.macro_fields,
        macro_mod.gen_reading(),
        macro_mod.gen_writing(),
    );

    let mod_gen = &macro_mod.mod_inner;
    let fields = fields.generate_fields();

    let attr = &mod_gen.attrs;
    let ident = &mod_gen.ident;
    let vis = &mod_gen.vis;
    let unsaftey = &mod_gen.unsafety;

    let mod_inner = &mod_gen
        .content
        .as_ref()
        .expect("Macro must contain content!")
        .1;

    quote! {
        #(#attr)*
        #vis #unsaftey mod #ident {
            #(#mod_inner)*

            #fields
        }
    }
}
