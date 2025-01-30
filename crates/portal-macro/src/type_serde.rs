/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

use proc_macro_error::emit_error;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, spanned::Spanned};

use crate::portal_parse::{EndpointKind, PortalEndpoint, PortalMacroInput};

fn to_module_name(ident: &Ident) -> Ident {
    let mut new_str = String::new();
    for old_char in ident.to_string().chars() {
        if old_char.is_uppercase() {
            if !new_str.is_empty() {
                new_str.push('_');
            }
            new_str.push(old_char.to_ascii_lowercase());
        } else {
            new_str.push(old_char);
        }
    }

    Ident::new(&new_str, ident.span())
}

pub fn generate_ast_portal(portal: &PortalMacroInput) -> TokenStream {
    let portal_name = &portal.trait_input.portal_name;
    let portal_module_name = to_module_name(portal_name);
    let portal_vis = &portal.trait_input.vis;

    let client_tokens = generate_client_trait(portal);

    quote! {
        #portal_vis mod #portal_module_name {
            /// Client side communication over this portal
            pub mod client {
                #client_tokens
            }

            /// Server side communication over this portal
            pub mod server {

            }
        }
    }
}

fn to_enum_name(fn_ident: &Ident) -> Ident {
    let mut new_str = String::new();
    let mut next_char_should_be_upper = true;

    for old_char in fn_ident.to_string().chars() {
        if old_char == '_' {
            next_char_should_be_upper = true;
            continue;
        }

        new_str.push(if next_char_should_be_upper {
            next_char_should_be_upper = false;
            old_char.to_ascii_uppercase()
        } else {
            old_char.to_ascii_lowercase()
        });
    }

    Ident::new(&new_str, fn_ident.span())
}

fn generate_endpoint_trait_sig(
    endpoint: &PortalEndpoint,
    portal_name: &Ident,
    means_fn: &Ident,
    input_enum_ident: &Ident,
    output_enum_ident: &Ident,
) -> TokenStream {
    let docs = &endpoint.docs;
    let ident = &endpoint.fn_ident;
    let unsafety = &endpoint.is_unsafe;
    let inputs = endpoint.input.iter().cloned();
    let outputs = &endpoint.output;

    let endpoint_enum_front = to_enum_name(ident);

    let input_argument_names = endpoint.input.iter().map(|e| match e {
        syn::FnArg::Typed(pat_type) => match pat_type.pat.as_ref() {
            syn::Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            _ => {
                emit_error!(
                    pat_type,
                    "This function argument input schema is not supported!"
                );
                Ident::new("not_supported", pat_type.span())
            }
        },
        syn::FnArg::Receiver(receiver) => Ident::new("not_supported_self", receiver.span()),
    });

    quote! {
        #(#docs)*
        #unsafety fn #ident(#(#inputs)*) #outputs {
            match #means_fn(#input_enum_ident::#endpoint_enum_front( #(#input_argument_names)* )) {
                #output_enum_ident::#endpoint_enum_front(inner_fn_value) => inner_fn_value,
                inner_fn_value => unreachable!("Got `{:?}`, but expected '{}' for {}'s endpoint {}",
                    inner_fn_value,
                    stringify!(#output_enum_ident::#endpoint_enum_front),
                    stringify!(#portal_name),
                    stringify!(#ident)
                ),
            }
        }
    }
}

/// Defines all the events
pub fn generate_client_trait(portal: &PortalMacroInput) -> TokenStream {
    let means_fn_ident = Ident::new("means_fn", portal.trait_input.portal_name.span());
    let input_enum_ident = Ident::new("KernelPortalInputs", portal.trait_input.portal_name.span());
    let output_enum_ident =
        Ident::new("KernelPortalOutputs", portal.trait_input.portal_name.span());

    let portal_name = &portal.trait_input.portal_name;
    let endpoints: Vec<TokenStream> = portal
        .trait_input
        .endpoints
        .iter()
        .filter(|e| matches!(e.endpoint, EndpointKind::Event(_)))
        .map(|endpoint| {
            generate_endpoint_trait_sig(
                endpoint,
                portal_name,
                &means_fn_ident,
                &input_enum_ident,
                &output_enum_ident,
            )
        })
        .collect();

    quote! {
        pub trait #portal_name {
            #(#endpoints)*

        }
    }
}
