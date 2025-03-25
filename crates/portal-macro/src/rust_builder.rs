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

use crate::ast;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use quote::TokenStreamExt;
use quote::format_ident;
use quote::quote;
use quote::quote_spanned;
use syn::Lifetime;

#[cfg(any(feature = "ipc-client", feature = "ipc-server"))]
use {
    crate::ast::ClientServerTokens, std::hash::DefaultHasher, std::hash::Hash, std::hash::Hasher,
};

/// A generator for the portal's trait
pub struct PortalTrait<'a> {
    portal: &'a ast::PortalMacro,
}

impl<'a> PortalTrait<'a> {
    pub fn new(portal: &'a ast::PortalMacro) -> Self {
        Self { portal }
    }
}

/// A generator for the portal's user defined types
pub struct PortalUserDefined<'a> {
    portal: &'a ast::PortalMacro,
}

impl<'a> PortalUserDefined<'a> {
    pub fn new(portal: &'a ast::PortalMacro) -> Self {
        Self { portal }
    }
}

/// A generator for the enum that all functions will put their arguments
#[cfg(any(feature = "syscall-client", feature = "syscall-server"))]
pub struct PortalTranslationInputType<'a> {
    portal: &'a ast::PortalMacro,
}

#[cfg(any(feature = "syscall-client", feature = "syscall-server"))]
impl<'a> PortalTranslationInputType<'a> {
    pub fn new(portal: &'a ast::PortalMacro) -> Self {
        Self { portal }
    }
}

/// A generator for the enum that all functions will output
#[cfg(any(feature = "syscall-client", feature = "syscall-server"))]
pub struct PortalTranslationOutputType<'a> {
    portal: &'a ast::PortalMacro,
}

#[cfg(any(feature = "syscall-client", feature = "syscall-server"))]
impl<'a> PortalTranslationOutputType<'a> {
    pub fn new(portal: &'a ast::PortalMacro) -> Self {
        Self { portal }
    }
}

/// A generator for a type that requires a lifetime
pub struct LifetimedProtocolVarType<'a> {
    lifetime_ident: &'a Lifetime,
    ty: &'a ast::ProtocolVarType,
}

impl<'a> LifetimedProtocolVarType<'a> {
    pub fn new(lifetime_ident: &'a Lifetime, ty: &'a ast::ProtocolVarType) -> Self {
        Self { lifetime_ident, ty }
    }
}

/// A generator for all the functions if they are intended to be global
#[cfg(any(feature = "syscall-client", feature = "syscall-server"))]
pub struct GlobalSyscallFunctionImpl<'a> {
    portal: &'a ast::PortalMacro,
}

#[cfg(any(feature = "syscall-client", feature = "syscall-server"))]
impl<'a> GlobalSyscallFunctionImpl<'a> {
    pub fn new(portal: &'a ast::PortalMacro) -> Self {
        Self { portal }
    }
}

/// A generator for QuantumOS's into syscall
/// (aka. The default type that will impl the portal's trait)
#[cfg(feature = "syscall-client")]
pub struct IntoSyscallPortalImpl<'a> {
    portal: &'a ast::PortalMacro,
}

#[cfg(feature = "syscall-client")]
impl<'a> IntoSyscallPortalImpl<'a> {
    pub fn new(portal: &'a ast::PortalMacro) -> Self {
        Self { portal }
    }
}

/// A generator for QuantumOS's out of syscall
#[cfg(feature = "syscall-server")]
pub struct OutSyscallPortalImpl<'a> {
    portal: &'a ast::PortalMacro,
}

#[cfg(feature = "syscall-server")]
impl<'a> OutSyscallPortalImpl<'a> {
    pub fn new(portal: &'a ast::PortalMacro) -> Self {
        Self { portal }
    }
}

#[cfg(feature = "ipc-client")]
pub struct PortalServerRequestEnum<'a> {
    portal: &'a ast::PortalMacro,
}

#[cfg(feature = "ipc-client")]
impl<'a> PortalServerRequestEnum<'a> {
    pub fn new(portal: &'a ast::PortalMacro) -> Self {
        Self { portal }
    }
}

#[cfg(feature = "ipc-server")]
pub struct PortalClientRequestEnum<'a> {
    portal: &'a ast::PortalMacro,
}

#[cfg(feature = "ipc-server")]
impl<'a> PortalClientRequestEnum<'a> {
    pub fn new(portal: &'a ast::PortalMacro) -> Self {
        Self { portal }
    }
}

#[cfg(any(feature = "ipc-client", feature = "ipc-server"))]
pub struct PortalInfoStruct<'a> {
    portal: &'a ast::PortalMacro,
}

#[cfg(any(feature = "ipc-client", feature = "ipc-server"))]
impl<'a> PortalInfoStruct<'a> {
    pub fn new(portal: &'a ast::PortalMacro) -> Self {
        Self { portal }
    }
}

/// Generate the Rust portal output tokens
pub fn generate_rust_portal(portal: &ast::PortalMacro) -> TokenStream2 {
    portal.to_token_stream()
}

impl ToTokens for ast::PortalMacro {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let portal_trait = PortalTrait::new(self);
        let user_defined = PortalUserDefined::new(self);

        tokens.append_all(quote! {
            #user_defined
            #portal_trait

        });

        if self.is_syscall_kind() {
            #[cfg(any(feature = "syscall-client", feature = "syscall-server"))]
            {
                let input = PortalTranslationInputType::new(self);
                let output = PortalTranslationOutputType::new(self);

                tokens.append_all(quote! {
                    #input
                    #output
                });
            }
            #[cfg(feature = "syscall-client")]
            {
                let into_portal_impl = IntoSyscallPortalImpl::new(self);
                let global_fn = GlobalSyscallFunctionImpl::new(self);

                tokens.append_all(quote! {
                    pub mod sys_client {
                        use super::*;

                        #into_portal_impl
                        #global_fn
                    }
                });
            }
            #[cfg(feature = "syscall-server")]
            {
                let out_portal_impl = OutSyscallPortalImpl::new(self);

                tokens.append_all(quote! {
                    pub mod sys_server {
                        use super::*;

                        #out_portal_impl
                    }
                });
            }
        } else {
            #[cfg(any(feature = "ipc-client", feature = "ipc-server"))]
            {
                let info_trait = PortalInfoStruct::new(self);

                info_trait.to_tokens(tokens);
            }
            #[cfg(feature = "ipc-client")]
            {
                let server_enum = PortalServerRequestEnum::new(self);

                server_enum.to_tokens(tokens);
            }
            #[cfg(feature = "ipc-server")]
            {
                let client_enum = PortalClientRequestEnum::new(self);

                client_enum.to_tokens(tokens);
            }
        }
    }
}

#[cfg(any(feature = "ipc-client", feature = "ipc-server"))]
impl<'a> ToTokens for PortalInfoStruct<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let portal_ident = self.portal.get_info_struct_ident();
        let endpoint_name = self.portal.get_service_name();

        let mut endpoint_hash = DefaultHasher::new();
        endpoint_name.hash(&mut endpoint_hash);
        let endpoint_hash = endpoint_hash.finish();

        tokens.append_all(quote! {
            pub struct #portal_ident(());

            impl ::portal::ipc::IpcServiceInfo for #portal_ident {
                const ENDPOINT_NAME: &'static str = #endpoint_name;
                const ENDPOINT_HASH: u64 = #endpoint_hash;
            }
        });
    }
}

#[cfg(feature = "ipc-client")]
impl<'a> ToTokens for PortalServerRequestEnum<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let server_enum = self.portal.get_output_enum_ident();
        let info_struct = self.portal.get_info_struct_ident();

        let server_requests = self
            .portal
            .endpoints
            .iter()
            .filter(|endpoint| endpoint.kind == ast::ProtocolEndpointKind::Handle)
            .map(|event| {
                let name = event.get_enum_ident();
                let target_id = event.portal_id.0 as u64;

                let type_body = if !event.is_async {
                    let output_type = &event.output_arg.0;

                    quote! {
                        {
                            sender: ::portal::ipc::IpcResponder<'sender, Glue, #info_struct, #output_type, #target_id>
                        }
                    }
                } else {
                    quote! {}
                };

                quote! {
                    #name #type_body
                }
            });

        tokens.append_all(quote! {
            #[non_exhaustive]
            pub enum #server_enum<'sender, Glue: ::portal::ipc::IpcGlue> {
                #[doc(hidden)]
                _Unused(::core::marker::PhantomData<&'sender Glue>),
                #(#server_requests),*
            }
        });
    }
}

#[cfg(feature = "ipc-server")]
impl<'a> ToTokens for PortalClientRequestEnum<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let client_enum = self.portal.get_input_enum_ident();
        let info_struct = self.portal.get_info_struct_ident();

        let client_requests = self
            .portal
            .endpoints
            .iter()
            .filter(|endpoint| endpoint.kind == ast::ProtocolEndpointKind::Event)
            .map(|event| {
                let name = event.get_enum_ident();
                let target_id = event.portal_id.0 as u64;

                let type_body = if !event.is_async {
                    let output_type = &event.output_arg.0;

                    quote! {
                        {
                            sender: ::portal::ipc::IpcResponder<'sender, Glue, #info_struct, #output_type, #target_id>
                        }
                    }
                } else {
                    quote! {}
                };

                quote! {
                    #name #type_body
                }
            });

        tokens.append_all(quote! {
            #[non_exhaustive]
            pub enum #client_enum<'sender, Glue: ::portal::ipc::IpcGlue> {
                #[doc(hidden)]
                _Unused(::core::marker::PhantomData<&'sender Glue>),
                #(#client_requests),*
            }
        });
    }
}

impl<'a> ToTokens for PortalTrait<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        #[cfg(any(feature = "syscall-client", feature = "syscall-server"))]
        if self.portal.is_syscall_kind() {
            let trait_ident = &self.portal.trait_ident;
            let endpoints = &self.portal.endpoints;

            tokens.append_all(quote! {
                pub trait #trait_ident {
                    #(#endpoints)*
                }
            });
        }

        #[cfg(feature = "ipc-client")]
        if !self.portal.is_syscall_kind() {
            let client_trait = self.portal.trait_client_name();
            let endpoints = self
                .portal
                .endpoints
                .iter()
                .map(|endpoint| endpoint.client_tokens());

            let client_enum = self.portal.get_output_enum_ident();
            let info_struct = self.portal.get_info_struct_ident();

            let target_tokens = self.portal.endpoints
                .iter()
                .filter(|endpoint| endpoint.kind == ast::ProtocolEndpointKind::Handle)
                .map(|endpoint| {
                    let target_id = endpoint.portal_id.0 as u64;
                    let enum_name = endpoint.get_enum_ident();

                    if endpoint.is_async {
                        quote!{
                            #target_id => return Ok(#client_enum::#enum_name),
                        }
                    } else {
                        quote!{
                            #target_id => return Ok(#client_enum::#enum_name { sender: ::portal::ipc::IpcResponder::new(&mut self.0)}),
                        }
                    }
                });

            tokens.append_all(quote! {
                pub struct #client_trait<Glue: ::portal::ipc::IpcGlue>(::portal::ipc::IpcService<Glue, #info_struct>);

                impl<Glue: ::portal::ipc::IpcGlue> #client_trait<Glue> {
                    pub fn new(glue: Glue) -> Self {
                        Self(::portal::ipc::IpcService::new(glue, false))
                    }

                    #(#endpoints)*
                    pub fn incoming<'a>(&'a mut self) -> ::portal::ipc::IpcResult<#client_enum<'a, Glue>> {
                        self.0.drive_rx()?;

                        let Some(ipc_msg) = self.0.pop_rx() else {
                            return Err(::portal::ipc::IpcError::NotReady);
                        };

                        match ipc_msg.target_id {
                            #(#target_tokens)*
                            _ => return Err(::portal::ipc::IpcError::InvalidTypeConvert),
                        }
                    }
                }
            });
        }

        #[cfg(feature = "ipc-server")]
        if !self.portal.is_syscall_kind() {
            let server_trait = self.portal.trait_server_name();
            let endpoints = self
                .portal
                .endpoints
                .iter()
                .map(|endpoint| endpoint.server_tokens());

            let server_enum = self.portal.get_input_enum_ident();
            let info_struct = self.portal.get_info_struct_ident();

            let target_tokens = self.portal.endpoints
                .iter()
                .filter(|endpoint| endpoint.kind == ast::ProtocolEndpointKind::Event)
                .map(|endpoint| {
                    let target_id = endpoint.portal_id.0 as u64;
                    let enum_name = endpoint.get_enum_ident();

                    if endpoint.is_async {
                        quote!{
                            #target_id => return Ok(#server_enum::#enum_name),
                        }
                    } else {
                        quote!{
                            #target_id => return Ok(#server_enum::#enum_name { sender: ::portal::ipc::IpcResponder::new(&mut self.0)}),
                        }
                    }
                });

            tokens.append_all(quote! {
                pub struct #server_trait<Glue: ::portal::ipc::IpcGlue>(::portal::ipc::IpcService<Glue, #info_struct>);

                impl<Glue: ::portal::ipc::IpcGlue> #server_trait<Glue> {
                    pub fn new(glue: Glue) -> Self {
                        Self(::portal::ipc::IpcService::new(glue, true))
                    }

                    #(#endpoints)*
                    pub fn incoming<'a>(&'a mut self) -> ::portal::ipc::IpcResult<#server_enum<'a, Glue>> {
                        self.0.drive_rx()?;

                        let Some(ipc_msg) = self.0.pop_rx() else {
                            return Err(::portal::ipc::IpcError::NotReady);
                        };

                        match ipc_msg.target_id {
                            #(#target_tokens)*
                            _ => return Err(::portal::ipc::IpcError::InvalidTypeConvert),
                        }
                    }
                }
            });
        }
    }
}

#[cfg(any(feature = "ipc-client", feature = "ipc-server"))]
impl ClientServerTokens for ast::ProtocolEndpoint {
    fn client_tokens(&self) -> TokenStream2 {
        match self.kind {
            ast::ProtocolEndpointKind::Event => {
                let output_ty = &self.output_arg.0;
                let docs = &self.doc_attributes;

                let fn_name = match &output_ty {
                    ast::ProtocolVarType::Unit(_) if self.is_async => {
                        format_ident!("{}_async", &self.fn_ident)
                    }
                    invalid_ty if self.is_async => {
                        return syn::Error::new(
                            invalid_ty.span(),
                            "`event` outputs are not supported with async requests!",
                        )
                        .to_compile_error()
                        .into();
                    }
                    _ => format_ident!("{}_blocking", &self.fn_ident),
                };

                let target_id = self.portal_id.0 as u64;
                let blocking_tokens = if !self.is_async {
                    quote! {
                        self.0.blocking_rx(#target_id)
                    }
                } else {
                    quote! {
                        Ok(())
                    }
                };

                quote! {
                    #(#docs)*
                    pub fn #fn_name(&mut self) -> ::portal::ipc::IpcResult<#output_ty> {
                        const TARGET_ID: u64 = #target_id;

                        self.0.tx_msg(TARGET_ID, false, ())?;
                        #blocking_tokens
                    }
                }
            }
            _ => quote! {},
        }
    }

    fn server_tokens(&self) -> TokenStream2 {
        match self.kind {
            ast::ProtocolEndpointKind::Handle => {
                let output_args = &self.output_arg;
                let docs = &self.doc_attributes;

                let fn_name = match &output_args.0 {
                    ast::ProtocolVarType::Unit(_) if self.is_async => {
                        format_ident!("ask_{}_async", &self.fn_ident)
                    }
                    invalid_ty if self.is_async => {
                        return syn::Error::new(
                            invalid_ty.span(),
                            "`handle` outputs are not supported with async requests!",
                        )
                        .to_compile_error()
                        .into();
                    }
                    _ => format_ident!("ask_{}_blocking", &self.fn_ident),
                };

                quote! {
                    #(#docs)*
                    pub fn #fn_name(&mut self) #output_args;
                }
            }
            _ => quote! {},
        }
    }
}

impl ToTokens for ast::ProtocolEndpoint {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let fn_ident = &self.fn_ident;
        let docs = &self.doc_attributes;
        let arguments = &self.input_args;
        let return_type = &self.output_arg;

        tokens.append_all(quote! {
            #(#docs)*
            fn #fn_ident(#(#arguments),*) #return_type;

        });
    }
}

impl ToTokens for ast::ProtocolInputArg {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let argument_name = &self.argument_ident;
        let ty = &self.ty;

        tokens.append_all(quote! {
            #argument_name : #ty
        });
    }
}

impl ToTokens for ast::ProtocolOutputArg {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let inner = &self.0;
        if !matches!(inner, ast::ProtocolVarType::Unit(_)) {
            tokens.append_all(quote! { -> #inner});
        }
    }
}

impl<'a> ToTokens for PortalUserDefined<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let user_defined_types = self
            .portal
            .endpoints
            .iter()
            .flat_map(|endpoint| endpoint.body.iter());
        tokens.append_all(quote! {
            #(#user_defined_types)*
        });
    }
}

impl ToTokens for ast::ProtocolDefine {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            ast::ProtocolDefine::DefinedEnum(ref_cell) => {
                let enum_def = ref_cell.borrow();

                let docs = &enum_def.docs;
                let ident = &enum_def.ident;
                let varients = &enum_def.varients;

                let lifetime = if enum_def.requires_lifetime {
                    quote! {<'defined>}
                } else {
                    quote! {}
                };

                tokens.append_all(quote! {
                    #(#docs)*
                    #[derive(Debug, Clone)]
                    pub enum #ident #lifetime {
                        #(#varients),*
                    }
                });
            }
            ast::ProtocolDefine::DefinedStruct(ref_cell) => {
                let struct_def = ref_cell.borrow();

                let docs = &struct_def.docs;
                let ident = &struct_def.ident;
                let items = &struct_def.items;

                if items.iter().any(|struct_field| struct_field.name.is_some()) {
                    // Named fields
                    tokens.append_all(quote! {
                        #(#docs)*
                        #[repr(C)]
                        #[derive(Debug, Clone)]
                        pub struct #ident {
                            #(#items),*
                        }
                    });
                } else {
                    // Unnamed fields
                    tokens.append_all(quote! {
                        #(#docs)*
                        #[repr(C)]
                        #[derive(Debug, Clone)]
                        pub struct #ident (#(#items),*);
                    });
                }
            }
        }
    }
}

impl ToTokens for ast::ProtocolStructItem {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let docs = &self.docs;
        let name = &self.name;
        let ty = &self.ty;

        if let Some(name) = name {
            tokens.append_all(quote! {
                #(#docs)*
                pub #name : #ty
            });
        } else {
            tokens.append_all(quote! {
                #(#docs)*
                pub #ty
            });
        }
    }
}

impl ToTokens for ast::ProtocolEnumVarient {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ident = &self.ident;
        let fields = &self.fields;
        let docs = &self.docs;

        tokens.append_all(quote! {
            #(#docs)*
            #ident #fields
        });
    }
}

impl ToTokens for ast::ProtocolEnumFields {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            ast::ProtocolEnumFields::None => {
                tokens.append_all(quote! {});
            }
            ast::ProtocolEnumFields::Unnamed(protocol_var_types) => {
                tokens.append_all(quote! {(#(#protocol_var_types),*)});
            }
            ast::ProtocolEnumFields::Named(hash_map) => {
                let var_defs = hash_map.iter().map(|(name, ty)| quote! { #name : #ty });

                tokens.append_all(quote! {
                    { #(#var_defs),* }
                });
            }
        }
    }
}

#[cfg(any(feature = "syscall-client", feature = "syscall-server"))]
impl<'a> ToTokens for PortalTranslationInputType<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let lifetime = Lifetime::new("'a", Span::call_site());

        let translation_ident = self.portal.get_input_enum_ident();
        let varients = self.portal.endpoints.iter().map(|endpoint| {
            let named_var = endpoint.input_args.iter().map(|input_arg| {
                let ty = LifetimedProtocolVarType::new(&lifetime, &input_arg.ty);
                let ident = &input_arg.argument_ident;

                quote! {
                    #ident : #ty
                }
            });
            let endpoint_enum_name = format_ident!("{}Endpoint", endpoint.get_enum_ident());

            let fields = if !endpoint.input_args.is_empty() {
                quote! { { #(#named_var),* } }
            } else {
                quote! {}
            };

            quote! {
                #endpoint_enum_name #fields,
            }
        });

        // TODO: We should try and not emit this field in the future, and look to see if we
        // actually need to use the lifetime.
        tokens.append_all(quote! {
            pub enum #translation_ident<#lifetime> {
                #(#varients)*
                _UnusedPhantomData(core::marker::PhantomData<&#lifetime ()>)
            }
        });
        tokens.append_all(quote! {
            unsafe impl<'input_lifetime> ::portal::syscall::SyscallInput for #translation_ident<'input_lifetime> {
                fn version_id() -> u32 {
                    1
                }
            }
        });
    }
}

#[cfg(any(feature = "syscall-client", feature = "syscall-server"))]
impl<'a> ToTokens for PortalTranslationOutputType<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let translation_ident = self.portal.get_output_enum_ident();
        let varients = self.portal.endpoints.iter().map(|endpoint| {
            let var_output = &endpoint.output_arg.0;
            let endpoint_enum_name = format_ident!("{}Endpoint", endpoint.get_enum_ident());

            let fields = if !matches!(var_output, ast::ProtocolVarType::Unit(_))
                && !matches!(var_output, ast::ProtocolVarType::Never(_))
            {
                quote! { ( #var_output ) }
            } else {
                quote! {}
            };

            quote! {
                #endpoint_enum_name #fields,
            }
        });

        tokens.append_all(quote! {
            pub enum #translation_ident {
                #(#varients)*
            }
        });
        tokens.append_all(quote! {
            unsafe impl ::portal::syscall::SyscallOutput for #translation_ident {
                fn version_id() -> u32 {
                    1
                }
            }
        });
    }
}

impl ToTokens for ast::ProtocolVarType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            ast::ProtocolVarType::ResultKind {
                span,
                ok_ty,
                err_ty,
            } => {
                tokens.append_all(quote_spanned! {span.clone()=>::core::result::Result});
                tokens.append_all(quote! {<#ok_ty, #err_ty>});
            }
            ast::ProtocolVarType::Never(span) => {
                tokens.append_all(quote_spanned! {span.clone()=>!})
            }
            ast::ProtocolVarType::Unit(span) => {
                tokens.append_all(quote_spanned! {span.clone()=>()})
            }
            ast::ProtocolVarType::Bool(span) => {
                tokens.append_all(quote_spanned! {span.clone()=>bool})
            }
            ast::ProtocolVarType::Signed8(span) => {
                tokens.append_all(quote_spanned! {span.clone()=>i8})
            }
            ast::ProtocolVarType::Signed16(span) => {
                tokens.append_all(quote_spanned! {span.clone()=>i16})
            }
            ast::ProtocolVarType::Signed32(span) => {
                tokens.append_all(quote_spanned! {span.clone()=>i32})
            }
            ast::ProtocolVarType::Signed64(span) => {
                tokens.append_all(quote_spanned! {span.clone()=>i64})
            }
            ast::ProtocolVarType::Unsigned8(span) => {
                tokens.append_all(quote_spanned! {span.clone()=>u8})
            }
            ast::ProtocolVarType::Unsigned16(span) => {
                tokens.append_all(quote_spanned! {span.clone()=>u16})
            }
            ast::ProtocolVarType::Unsigned32(span) => {
                tokens.append_all(quote_spanned! {span.clone()=>u32})
            }
            ast::ProtocolVarType::Unsigned64(span) => {
                tokens.append_all(quote_spanned! {span.clone()=>u64})
            }
            ast::ProtocolVarType::UnsignedSize(span) => {
                tokens.append_all(quote_spanned! {span.clone()=>usize})
            }
            ast::ProtocolVarType::Unknown(ident) => {
                let error_msg = format!("Unknown Type '{}'", ident.to_string());
                tokens.append_all(quote_spanned! {ident.span()=>compile_error!(#error_msg)})
            }
            ast::ProtocolVarType::UserDefined { span, to } => {
                let type_ident = to.var_ident();
                tokens.append_all(quote_spanned! {span.clone()=>#type_ident})
            }
            ast::ProtocolVarType::Str(span) => {
                tokens.append_all(quote_spanned! {span.clone()=>str})
            }
            ast::ProtocolVarType::RefTo { span, is_mut, to } if *is_mut => {
                tokens.append_all(quote_spanned! {span.clone()=>&mut #to})
            }
            ast::ProtocolVarType::RefTo { span, to, .. } => {
                tokens.append_all(quote_spanned! {span.clone()=>&#to})
            }
            ast::ProtocolVarType::PtrTo { span, is_mut, to } if *is_mut => {
                tokens.append_all(quote_spanned! {span.clone()=>*mut #to})
            }
            ast::ProtocolVarType::PtrTo { span, to, .. } => {
                tokens.append_all(quote_spanned! {span.clone()=>*const #to})
            }
            ast::ProtocolVarType::Array { span, to, len } => {
                if let Some(len) = len {
                    let array_inner = quote! { #to; #len };
                    tokens.append_all(quote_spanned! {span.clone()=>[#array_inner]});
                } else {
                    tokens.append_all(quote! {[#to]});
                }
            }
            ast::ProtocolVarType::IpcString(span) => {
                tokens.append_all(quote_spanned! {span.clone()=> ::portal::ipc::IpcString });
            }
            ast::ProtocolVarType::IpcVec { span, to } => {
                tokens.append_all(quote_spanned! {span.clone()=> ::portal::ipc::IpcVec});
                tokens.append_all(quote! {<#to>});
            }
        }
    }
}

impl<'a> ToTokens for LifetimedProtocolVarType<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.ty {
            ast::ProtocolVarType::ResultKind {
                span,
                ok_ty,
                err_ty,
            } => {
                let ok_ty = Self::new(&self.lifetime_ident, &ok_ty);
                let err_ty = Self::new(&self.lifetime_ident, &err_ty);

                tokens.append_all(quote_spanned! {span.clone()=>::core::result::Result});
                tokens.append_all(quote! {<#ok_ty, #err_ty>});
            }
            ast::ProtocolVarType::RefTo { span, is_mut, to } => {
                let lifetime = self.lifetime_ident;
                tokens.append_all(quote_spanned! {span.clone()=>&});
                tokens.append_all(quote! {#lifetime});

                if *is_mut {
                    tokens.append_all(quote! {mut});
                }

                let to = Self::new(&self.lifetime_ident, &to);
                tokens.append_all(quote! { #to });
            }
            ast::ProtocolVarType::PtrTo { span, is_mut, to } => {
                tokens.append_all(quote_spanned! {span.clone()=>*});
                if *is_mut {
                    tokens.append_all(quote! {mut});
                } else {
                    tokens.append_all(quote! {const});
                }

                let to = Self::new(&self.lifetime_ident, &to);
                tokens.append_all(quote! { #to });
            }
            ty => ty.to_tokens(tokens),
        }
    }
}

#[cfg(feature = "syscall-client")]
impl<'a> ToTokens for IntoSyscallPortalImpl<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ident = self.portal.trait_client_name();
        let trait_ident = &self.portal.trait_ident;
        let input_enum = self.portal.get_input_enum_ident();
        let output_enum = self.portal.get_output_enum_ident();

        let endpoints = self.portal.endpoints.iter().map(|endpoint| {
            let fn_ident = &endpoint.fn_ident;
            let docs = &endpoint.doc_attributes;
            let arguments = &endpoint.input_args;
            let return_type = &endpoint.output_arg;
            let enum_part = format_ident!("{}Endpoint", endpoint.get_enum_ident());

            let syscall_part = if arguments.len() > 0 {
                let argument_in_body = endpoint.input_args.iter().map(|input_arg| {
                    let name = &input_arg.argument_ident;
                   quote! { #name }
                });

                quote! { match (unsafe { Self::call_syscall(#input_enum::#enum_part {#(#argument_in_body),*}) }) }
            } else {
                quote! { match (unsafe { Self::call_syscall(#input_enum::#enum_part) }) }
            };

            let fn_closing = match endpoint.output_arg.0 {
                ast::ProtocolVarType::Never(_) => {
                    let fmt_string = format!("Portal Endpoint '{}' promised to never return, but yet returned!", fn_ident);
                    let error_string = format!("Portal Endpoint '{}': '{}::call_syscall' was supposed to return '{}::{}'", fn_ident, ident, output_enum, enum_part);
                    quote! {
                        {
                            #output_enum::#enum_part => { unreachable!(#fmt_string); }
                            _ => {
                                unreachable!(#error_string)
                            }
                        };
                    }
                },
                ast::ProtocolVarType::Unit(_) => {
                    let fmt_string = format!("Portal Endpoint '{}': '{}::call_syscall' was supposed to return '{}::{}'", fn_ident, ident, output_enum, enum_part);
                    quote! {
                        {
                            #output_enum::#enum_part => (),
                            _ => {
                                unreachable!(#fmt_string)
                            }
                        }
                    }
                }
                _ => {
                    let fmt_string = format!("Portal Endpoint '{}': '{}::call_syscall' was supposed to return '{}::{}'", fn_ident, ident, output_enum, enum_part);
                    quote! {
                        {
                            #output_enum::#enum_part (output_val) => { output_val }
                            _ => {
                                unreachable!(#fmt_string)
                            }
                        }
                    }
                }
            };

            // TODO: In the future we should try and reduce the need to put values into
            //       the input argument enum, and instead try and serialize the values
            //       into the ~6 CPU registers we have for syscalls. It would improve
            //       performance, and be easier for C to call.
            quote! {
                #(#docs)*
                fn #fn_ident(#(#arguments),*) #return_type {
                    #syscall_part
                    #fn_closing
                }
            }
        });

        tokens.append_all(quote! {
            pub struct #ident {}
        });
        tokens.append_all(quote! {
            impl #trait_ident for #ident {
                #(#endpoints)*
            }
        });
        tokens.append_all(quote! {
            impl #ident {
                #[inline]
                unsafe fn call_syscall<'syscall>(arguments: #input_enum<'syscall>) -> #output_enum {
                    let mut output = unsafe { <#output_enum as ::portal::syscall::SyscallOutput>::before_call() };
                    ::portal::syscall::client::call_syscall(&arguments, &mut output);

                    output
                }
            }
        });
    }
}

#[cfg(any(feature = "syscall-client", feature = "syscall-server"))]
impl<'a> ToTokens for GlobalSyscallFunctionImpl<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if !self.portal.args.as_ref().is_some_and(|arg| arg.is_global) {
            return;
        }
        let into_impl = self.portal.trait_client_name();
        let trait_ident = &self.portal.trait_ident;

        let endpoint_fn = self.portal.endpoints.iter().map(|endpoint| {
            let fn_ident = &endpoint.fn_ident;
            let docs = &endpoint.doc_attributes;
            let arguments = &endpoint.input_args;
            let return_type = &endpoint.output_arg;

            let argument_in_body = endpoint.input_args.iter().map(|input_arg| {
                let name = &input_arg.argument_ident;
                quote! { #name }
            });

            quote! {
                #(#docs)*
                #[inline]
                pub fn #fn_ident(#(#arguments),*) #return_type {
                    <#into_impl as super::#trait_ident>::#fn_ident(#(#argument_in_body),*)
                }
            }
        });

        tokens.append_all(quote! {
            #(#endpoint_fn)*
        });
    }
}

#[cfg(feature = "syscall-server")]
impl<'a> ToTokens for OutSyscallPortalImpl<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let output_ident = self.portal.trait_server_name();
        let trait_ident = &self.portal.trait_ident;

        let input_enum = self.portal.get_input_enum_ident();
        let output_enum = self.portal.get_output_enum_ident();

        let endpoints = self.portal.endpoints.iter().map(|endpoint| {
            let enum_part = format_ident!("{}Endpoint", endpoint.get_enum_ident());
            let fn_ident = &endpoint.fn_ident;

            let (enum_args, function_args) = if endpoint.input_args.len() > 0 {
                let argument_in_body: Vec<_> = endpoint.input_args.iter().map(|input_arg| {
                    let name = &input_arg.argument_ident;
                   quote! { #name }
                }).collect();

                (quote! { { #(#argument_in_body),* } }, quote! { ( #(#argument_in_body),* ) })
            } else {
                (quote!{}, quote!{ () })
            };

            let output = if !matches!(endpoint.output_arg.0, ast::ProtocolVarType::Never(_)) && !matches!(endpoint.output_arg.0, ast::ProtocolVarType::Unit(_)) {
                quote!{{
                   super::#output_enum::#enum_part (<Self as #trait_ident>::#fn_ident #function_args)
                }}
            } else {
                quote! {{
                   <Self as #trait_ident>::#fn_ident #function_args;
                   super::#output_enum::#enum_part
                }}
            };

           quote!{
               super::#input_enum::#enum_part #enum_args => #output
           }
        });

        tokens.append_all(quote!{
            pub trait #output_ident : #trait_ident {
                #[inline]
                #[allow(unreachable_code)]
                unsafe fn from_syscall(kind: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
                    let syscall_input_ptr = arg0 as *const super::#input_enum;
                    let syscall_output_ptr = arg1 as *mut super::#output_enum;
                    let syscall_packed_len = arg2;
                    let syscall_packed_id = arg3;

                    if !<Self as #output_ident>::verify_user_ptr(syscall_input_ptr) || !<Self as #output_ident>::verify_user_ptr(syscall_output_ptr) {
                        return portal::syscall::SYSCALL_BAD_RESP;
                    }

                    unsafe {
                        ::portal::syscall::server::adapt_syscall(kind, syscall_input_ptr, syscall_output_ptr, syscall_packed_len, syscall_packed_id, |input| {
                            match input {
                                #(#endpoints)*
                                _ => unreachable!("Should never get here?"),
                            }
                        })
                    }
                }

                /// Check that the user's ptr is correct
                fn verify_user_ptr<T: Sized>(ptr: *const T) -> bool;
            }
        });
    }
}
