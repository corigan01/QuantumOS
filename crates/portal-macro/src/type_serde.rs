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
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{FnArg, Ident, Lifetime, Visibility, spanned::Spanned, token::Pub};

use crate::portal_parse::{EndpointKind, PortalEndpoint, PortalMacroInput, ProtocolKind};

#[derive(Debug)]
struct PortalMetadata {
    input_needs_lifetime: bool,
    input_types: Vec<(Ident, Vec<Box<syn::Type>>)>,
    output_type: Vec<(Ident, Box<syn::Type>)>,
}

impl PortalMetadata {
    pub fn new(portal: &PortalMacroInput) -> Self {
        let input_types: Vec<(Ident, Vec<Box<syn::Type>>)> = portal
            .trait_input
            .endpoints
            .iter()
            .map(|endpoint| {
                (
                    to_enum_name(&endpoint.fn_ident),
                    endpoint
                        .input
                        .iter()
                        .map(|input| -> Box<syn::Type> {
                            match input.clone() {
                                FnArg::Receiver(_) => Box::new(syn::parse_str("()").unwrap()),
                                FnArg::Typed(mut pat_type) => {
                                    match pat_type.ty.as_mut() {
                                        syn::Type::Reference(type_reference) => {
                                            type_reference.lifetime =
                                                Some(Lifetime::new("'a", Span::call_site()));
                                        }
                                        _ => (),
                                    }

                                    pat_type.ty
                                }
                            }
                        })
                        .collect(),
                )
            })
            .collect();

        let output_type = portal
        .trait_input
        .endpoints
        .iter()
        .map(|endpoint| {
            (
                to_enum_name(&endpoint.fn_ident),
                match endpoint.output.clone() {
                    syn::ReturnType::Default => Box::new(syn::parse_str("()").unwrap()),
                    syn::ReturnType::Type(_, mut ty) => {
                        match ty.as_mut() {
                            syn::Type::Reference(type_reference) => {
                                emit_error!(
                                    type_reference.span(),
                                    "Values with lifetimes are not supported in endpoint's output"
                                );
                            }
                            _ => (),
                        }

                        ty
                    }
                },
            )
        }).collect();

        let mut input_needs_lifetime = false;
        input_types.iter().for_each(|(_, ty_vec)| {
            ty_vec.iter().for_each(|typ| match typ.as_ref() {
                syn::Type::Reference(_) => input_needs_lifetime = true,
                _ => (),
            })
        });

        Self {
            input_needs_lifetime,
            input_types,
            output_type,
        }
    }
}

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

pub fn generate_ast_portal(portal: &PortalMacroInput) -> TokenStream2 {
    let metadata = PortalMetadata::new(portal);
    let portal_name = &portal.trait_input.portal_name;
    let portal_module_name = to_module_name(portal_name);
    let portal_vis = &portal.trait_input.vis;

    let input_enum_ident = Ident::new(
        &format!("{}Inputs", portal.trait_input.portal_name.to_string()),
        portal.trait_input.portal_name.span(),
    );
    let output_enum_ident = Ident::new(
        &format!("{}Output", portal.trait_input.portal_name.to_string()),
        portal.trait_input.portal_name.span(),
    );

    let trait_tokens =
        generate_client_trait(portal, &input_enum_ident, &output_enum_ident, &metadata);
    let enums = generate_endpoint_enums(&input_enum_ident, &output_enum_ident, &metadata);
    let defined_types = generate_functions_inner_types(&portal);
    let consts = generate_endpoint_consts(portal);

    quote! {
        #portal_vis mod #portal_module_name {
            #consts
            #defined_types
            #enums

            #trait_tokens
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

    new_str.push_str("Portal");
    Ident::new(&new_str, fn_ident.span())
}

fn endpoints_enum_input(endpoint: &PortalEndpoint, input_enum_ident: &Ident) -> TokenStream2 {
    let ident = &endpoint.fn_ident;
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

    if endpoint.input.is_empty() {
        quote! {
            #input_enum_ident::#endpoint_enum_front
        }
    } else {
        quote! {
            #input_enum_ident::#endpoint_enum_front( #(#input_argument_names),* )
        }
    }
}

fn generate_functions_inner_types(portal: &PortalMacroInput) -> TokenStream2 {
    let portal_types = portal
        .trait_input
        .endpoints
        .iter()
        .map(|endpoint| {
            endpoint.fn_body.stmts.iter().map(|st| match st {
                syn::Stmt::Item(item) => match item {
                    syn::Item::Enum(item_enum) => {
                        let mut item_enum = item_enum.clone();
                        item_enum.vis = Visibility::Public(Pub {
                            span: item_enum.span(),
                        });

                        quote! {
                            /// Part of an endpoint
                            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
                            #item_enum
                        }
                    }
                    unsupported => {
                        emit_error!(
                            unsupported.span(),
                            "Unsupported definition, currently `#[portal]` only supports `enum`"
                        );
                        quote! {}
                    }
                },
                unsupported => {
                    emit_error!(
                        unsupported.span(),
                        "Unsupported definition, currently `#[portal]` only supports `enum`"
                    );
                    quote! {}
                }
            })
        })
        .flatten();

    quote! { #(#portal_types)* }
}

fn generate_endpoint_fn_body_syscall(
    endpoint: &PortalEndpoint,
    portal_name: &Ident,
    means_fn: &Ident,
    input_enum_ident: &Ident,
    output_enum_ident: &Ident,
) -> TokenStream2 {
    let ident = &endpoint.fn_ident;
    let endpoint_enum_front = to_enum_name(ident);
    let input_enum = endpoints_enum_input(endpoint, input_enum_ident);

    quote! {match Self::#means_fn(#input_enum) {
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

fn generate_endpoint_fn_body_ipc(
    endpoint: &PortalEndpoint,
    portal_name: &Ident,
    means_fn: &Ident,
    input_enum_ident: &Ident,
    output_enum_ident: &Ident,
) -> TokenStream2 {
    let ident = &endpoint.fn_ident;
    let endpoint_enum_front = to_enum_name(ident);
    let input_enum = endpoints_enum_input(endpoint, input_enum_ident);

    quote! {match self.#means_fn(#input_enum) {
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

fn generate_endpoint_trait_sig(
    endpoint: &PortalEndpoint,
    portal_name: &Ident,
    means_fn: &Ident,
    input_enum_ident: &Ident,
    output_enum_ident: &Ident,
    portal_kind: ProtocolKind,
) -> TokenStream2 {
    let docs = &endpoint.docs;
    let ident = &endpoint.fn_ident;
    let unsafety = &endpoint.is_unsafe;
    let inputs: Vec<FnArg> = match portal_kind {
        ProtocolKind::Ipc(_) => {
            let self_tokens: syn::FnArg = syn::parse_str("&self").unwrap();
            [self_tokens]
                .into_iter()
                .chain(endpoint.input.iter().cloned())
                .collect()
        }
        _ => endpoint.input.iter().cloned().collect(),
    };
    let outputs = &endpoint.output;

    let fn_body = match portal_kind {
        ProtocolKind::Unknown(_) => quote! {},
        ProtocolKind::Syscall(_) => generate_endpoint_fn_body_syscall(
            endpoint,
            portal_name,
            means_fn,
            input_enum_ident,
            output_enum_ident,
        ),
        ProtocolKind::Ipc(_) => generate_endpoint_fn_body_ipc(
            endpoint,
            portal_name,
            means_fn,
            input_enum_ident,
            output_enum_ident,
        ),
    };

    quote! {
        #(#docs)*
        #unsafety fn #ident(#(#inputs),*) #outputs {
            #fn_body
        }
    }
}

fn to_const_name(fn_ident: &Ident, extra: &str) -> Ident {
    let mut new_str = String::new();

    for old_char in fn_ident.to_string().chars() {
        new_str.push(old_char.to_ascii_uppercase());
    }

    new_str.push_str("_ID_");
    new_str.push_str(extra);
    Ident::new(&new_str, fn_ident.span())
}

fn generate_endpoint_consts(portal: &PortalMacroInput) -> TokenStream2 {
    let consts = portal
        .trait_input
        .endpoints
        .iter()
        .map(|endpoint| match &endpoint.endpoint {
            EndpointKind::None => quote! {},
            EndpointKind::Event(event_attribute) => {
                let id = event_attribute.id;
                let ident = to_const_name(&endpoint.fn_ident, "EVENT");

                quote! {
                    pub const #ident: usize = #id;
                }
            }
            EndpointKind::Handle(handle_attribute) => {
                let id = handle_attribute.id;
                let ident = to_const_name(&endpoint.fn_ident, "HANDLE");

                quote! {
                    pub const #ident: usize = #id;
                }
            }
        });

    quote! { #(#consts)* }
}

fn generate_endpoint_enums(
    input_enum_ident: &Ident,
    output_enum_ident: &Ident,
    metadata: &PortalMetadata,
) -> TokenStream2 {
    let all_input_types = metadata.input_types.iter().map(|(ident, arguments)| {
        if arguments.is_empty() {
            quote_spanned! {ident.span()=> #ident }
        } else {
            quote_spanned! {ident.span()=> #ident(#(#arguments),*) }
        }
    });

    let all_output_types = metadata.output_type.iter().map(|(ident, argument)| {
        quote_spanned! {ident.span()=> #ident(#argument) }
    });

    let input_enum_sig = if metadata.input_needs_lifetime {
        quote_spanned! {input_enum_ident.span()=> pub enum #input_enum_ident<'a> }
    } else {
        quote_spanned! {input_enum_ident.span()=> pub enum #input_enum_ident }
    };

    quote! {
        #[derive(Debug)]
        #input_enum_sig {
            #(#all_input_types),*
        }

        #[derive(Debug)]
        pub enum #output_enum_ident {
            #(#all_output_types),*
        }
    }
}

fn generate_into_portal_function(
    into_portal_ident: &Ident,
    input_enum_ident: &Ident,
    output_enum_ident: &Ident,
    metadata: &PortalMetadata,
    portal_kind: ProtocolKind,
) -> TokenStream2 {
    match portal_kind {
        ProtocolKind::Unknown(_) => quote! {},
        ProtocolKind::Syscall(_) if metadata.input_needs_lifetime => {
            quote! {
                fn #into_portal_ident<'a>(input: #input_enum_ident<'a>) -> #output_enum_ident;
            }
        }
        ProtocolKind::Syscall(_) => {
            quote! {
                fn #into_portal_ident(input: #input_enum_ident) -> #output_enum_ident;
            }
        }
        ProtocolKind::Ipc(_) if metadata.input_needs_lifetime => {
            quote! {
                fn #into_portal_ident<'a>(&self, input: #input_enum_ident<'a>) -> #output_enum_ident;
            }
        }
        ProtocolKind::Ipc(_) => quote! {
                fn #into_portal_ident(&self, input: #input_enum_ident) -> #output_enum_ident;
        },
    }
}

/// Defines all the events
fn generate_client_trait(
    portal: &PortalMacroInput,
    input_enum_ident: &Ident,
    output_enum_ident: &Ident,
    metadata: &PortalMetadata,
) -> TokenStream2 {
    let protocol_kind = portal.args.protocol;
    let into_portal_ident = Ident::new("into_portal", portal.trait_input.portal_name.span());
    let portal_name = &portal.trait_input.portal_name;
    let endpoints: Vec<TokenStream2> = portal
        .trait_input
        .endpoints
        .iter()
        .filter(|e| matches!(e.endpoint, EndpointKind::Event(_)))
        .map(|endpoint| {
            generate_endpoint_trait_sig(
                endpoint,
                portal_name,
                &into_portal_ident,
                &input_enum_ident,
                &output_enum_ident,
                protocol_kind.clone(),
            )
        })
        .collect();

    let into_portal_fn = generate_into_portal_function(
        &into_portal_ident,
        input_enum_ident,
        output_enum_ident,
        metadata,
        portal.args.protocol,
    );

    quote! {
        pub trait #portal_name {
            #into_portal_fn

            #(#endpoints)*

        }
    }
}
