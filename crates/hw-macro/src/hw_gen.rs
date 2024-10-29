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
    let our_provider = providers.get(&field.args.parent.as_ref().unwrap().to_string());
    quote! {}
}

pub fn gen(input: HwDeviceMacro) -> TokenStream {
    let providers = inspect_providers(&input);

    let mut token_mass = Vec::<TokenStream>::new();

    for mod_provider in &input.providers {
        let provider = &mod_provider.module;

        token_mass.push(quote! {
            #provider
        });
    }

    for field in &input.fields {
        token_mass.push(visit_field(&providers, field));
    }

    quote! {
        #(#token_mass)*
    }
}
