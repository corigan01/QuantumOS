use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;

use crate::hw_parse::{HwDeviceMacro, MacroFields, MacroProviders};

type ProviderMap<'a> = HashMap<String, &'a MacroProviders>;

fn inspect_providers<'a>(input: &'a HwDeviceMacro) -> ProviderMap<'a> {
    let mut map = HashMap::new();

    for mod_provider in &input.providers {
        map.insert(mod_provider.module.ident.to_string(), mod_provider);
    }

    map
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
    quote! {}
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
