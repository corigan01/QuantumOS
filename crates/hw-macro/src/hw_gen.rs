use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

use crate::hw_parse::{Access, Bits, HwDeviceMacro, MacroFields, MacroProviders};

struct ProviderInfo<'a> {
    provider: &'a MacroProviders,

    readable_bits: Option<usize>,
    writeable_bits: Option<usize>,
    can_read_be_const: bool,
    can_write_be_const: bool,
}

impl<'a> ProviderInfo<'a> {
    fn convert_type_to_bits(bit_type: &Type) -> Option<usize> {
        todo!("{:#?}", bit_type)
    }

    fn new(provider: &'a MacroProviders) -> Self {
        let readable_bits = provider
            .read_type
            .as_ref()
            .and_then(|read_type| Self::convert_type_to_bits(read_type));
        let writeable_bits = provider
            .write_type
            .as_ref()
            .and_then(|write_type| Self::convert_type_to_bits(write_type));

        let can_read_be_const = provider
            .fn_def
            .read_fn
            .as_ref()
            .is_some_and(|read_fn| read_fn.sig.constness.is_some());
        let can_write_be_const = provider
            .fn_def
            .read_fn
            .as_ref()
            .is_some_and(|read_fn| read_fn.sig.constness.is_some());

        Self {
            provider,
            readable_bits,
            writeable_bits,
            can_read_be_const,
            can_write_be_const,
        }
    }
}

// TODO: We should try and avoid using a Hashmap here
type ProviderKey = String;
type ProviderMap<'a> = HashMap<ProviderKey, ProviderInfo<'a>>;

fn inspect_providers<'a>(input: &'a HwDeviceMacro) -> ProviderMap<'a> {
    let mut map = HashMap::new();

    for mod_provider in &input.providers {
        map.insert(
            mod_provider.module.ident.to_string(),
            ProviderInfo::new(mod_provider),
        );
    }

    map
}

fn make_const_ident(ident: &Ident, post: &str) -> Ident {
    Ident::new(
        &format!(
            "{}{}",
            ident
                .to_string()
                .chars()
                .into_iter()
                .map(|c| {
                    match c {
                        ' ' => '_',
                        c if c.is_ascii_lowercase() => c.to_ascii_uppercase(),
                        c => c,
                    }
                })
                .collect::<String>(),
            post
        ),
        ident.span(),
    )
}

fn gen_read(read_value_tokens: TokenStream, field: &MacroFields) -> TokenStream {
    let ident_placeholder = make_const_ident(&field.ident, "_BIT");
    quote! {
        const #ident_placeholder: () = ();
    }
}

fn visit_field(providers: &ProviderMap, field: &MacroFields) -> TokenStream {
    let Some(our_provider) = field
        .args
        .parent
        .as_ref()
        .and_then(|parent_ident| providers.get(&parent_ident.to_string()))
    else {
        field
            .span
            .unwrap()
            .error("Parent could not be found. ")
            .help(format!(
                "Valid parents are: {}",
                providers.keys().fold(String::new(), |mut old, new| {
                    if !old.is_empty() {
                        old.push_str(", ");
                    }
                    old.push('\'');
                    old.push_str(new);
                    old.push('\'');

                    old
                })
            ))
            .emit();
        return quote! {};
    };

    match field.args.access {
        Access::RO => gen_read(quote! {}, field),
        _ => todo!(),
    }
}

pub fn gen(input: HwDeviceMacro) -> TokenStream {
    let providers = inspect_providers(&input);

    let mut token_mass = Vec::<TokenStream>::new();

    for mod_provider in &input.providers {
        token_mass.push(gen_module_provider(mod_provider));
    }

    for field in &input.fields {
        token_mass.push(visit_field(&providers, field));
    }

    quote! {
        #(#token_mass)*
    }
}

fn gen_module_provider(provider: &MacroProviders) -> TokenStream {
    let provider = &provider.module;

    // TODO: We will need much more complex bahavior in the future
    quote! {
        #provider
    }
}
