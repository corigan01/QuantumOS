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

use proc_macro_error::emit_error;
use quote::{format_ident, quote, quote_spanned};
use syn::{FnArg, Ident, Signature, spanned::Spanned};

use crate::parse::{ParsedInterrupt, ParsedInterruptArgs};

pub fn gen_interrupt(
    args: ParsedInterruptArgs,
    input: ParsedInterrupt,
) -> proc_macro2::TokenStream {
    verify_function_sig(&input.fn_tokens.sig);
    let fn_ident = &input.fn_tokens.sig.ident;

    let main_fn = gen_main_fn(&input);
    let (fn_tokens, no_error, error, reserved) = gen_macro_functions(&args, fn_ident);
    let consts = gen_consts(&args, no_error, error, reserved);

    let vis_tokens = &input.fn_tokens.vis;
    quote_spanned! {input.fn_tokens.span()=>
        #[allow(unused)]
        #vis_tokens mod #fn_ident {
            use super::*;
            #(#fn_tokens)*

            #consts
            #main_fn
        }
    }
}

fn gen_main_fn(input: &ParsedInterrupt) -> proc_macro2::TokenStream {
    let fn_body = &input.fn_tokens;
    quote_spanned! {input.fn_tokens.span()=>
        #fn_body
    }
}

fn gen_consts(
    fn_range: &ParsedInterruptArgs,
    no_error_fn_idents: Vec<(u8, Ident)>,
    error_fn_idents: Vec<(u8, Ident)>,
    reserved_fn_idents: Vec<(u8, Ident)>,
) -> proc_macro2::TokenStream {
    let ne_amount = no_error_fn_idents.len();
    let e_amount = error_fn_idents.len();
    let r_amount = reserved_fn_idents.len();

    let no_error_fns = no_error_fn_idents
        .into_iter()
        .map(|(irq, ident)| quote! {(#irq, #ident)});
    let error_fns = error_fn_idents
        .into_iter()
        .map(|(irq, ident)| quote! {(#irq, #ident)});
    let reserved_fns = reserved_fn_idents
        .into_iter()
        .map(|(irq, ident)| quote! {(#irq, #ident)});

    quote_spanned! {fn_range.span()=>
        pub const IRQ_FUNCTION_NE_PTRS: [(u8, ::arch::idt64::RawHandlerFuncNe); #ne_amount] = [#(#no_error_fns,)*];
        pub const IRQ_FUNCTION_E_PTRS: [(u8, ::arch::idt64::RawHandlerFuncE); #e_amount] = [#(#error_fns,)*];
        pub const IRQ_FUNCTION_RESERVED_PTRS: [(u8, ::arch::idt64::RawHandlerFuncNe); #r_amount] = [#(#reserved_fns,)*];
    }
}

fn gen_macro_functions(
    fn_range: &ParsedInterruptArgs,
    call_ident: &Ident,
) -> (
    Vec<proc_macro2::TokenStream>,
    Vec<(u8, Ident)>,
    Vec<(u8, Ident)>,
    Vec<(u8, Ident)>,
) {
    let (irq_s, irq_e) = fn_range.range();

    let mut tokens: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut no_error = Vec::new();
    let mut error = Vec::new();
    let mut reserved = Vec::new();

    for irq_id in irq_s as u8..=(irq_e as u8) {
        match irq_id {
            // No Error
            0..=7 | 9 | 16 | 18..=20 | 28 | 32..=255 => {
                let ident = format_ident!("irq_raw_{}_{irq_id}_ne", call_ident.to_string());
                no_error.push((irq_id, ident.clone()));

                tokens.push(quote_spanned! {fn_range.span()=>
                    extern "x86-interrupt" fn #ident(frame: ::arch::idt64::InterruptFrame) {
                        let int_info = ::arch::idt64::InterruptInfo::convert_from_ne(#irq_id, frame);
                        #call_ident(&int_info);
                    }
                });
            }
            // Yes Error
            8 | 10..=14 | 17 | 21 | 29 | 30 => {
                let ident = format_ident!("irq_raw_{}_{irq_id}_e", call_ident.to_string());
                error.push((irq_id, ident.clone()));

                tokens.push(quote_spanned! {fn_range.span()=>
                    extern "x86-interrupt" fn #ident(frame: ::arch::idt64::InterruptFrame, error: u64) {
                        let int_info = ::arch::idt64::InterruptInfo::convert_from_e(#irq_id, frame, error);
                        #call_ident(&int_info);
                    }
                });
            }
            // Reserved
            _ => {
                let ident = format_ident!("irq_raw_{}_{irq_id}_reserved", call_ident.to_string());
                reserved.push((irq_id, ident.clone()));

                tokens.push(quote_spanned! {fn_range.span()=>
                    extern "x86-interrupt" fn #ident(frame: ::arch::idt64::InterruptFrame) {
                        panic!("Reserved interrupt (IRQ{}) called! -- {:#?}", #irq_id, frame);
                    }
                });
            }
        }
    }

    (tokens, no_error, error, reserved)
}

fn verify_function_sig(fn_sig: &Signature) {
    if let Some(const_span) = fn_sig.constness {
        emit_error!(
            const_span.span(),
            "Interrupt function signature should not be 'const'";
            help = "Remove 'const'";
        );
    }
    if let Some(async_span) = fn_sig.asyncness {
        emit_error!(
            async_span.span(),
            "Interrupt function signature should not be 'async'";
            help = "Remove 'async'";
        );
    }

    fn_sig.generics.lifetimes().for_each(|lifetime_param| {
        emit_error!(
            lifetime_param.span(),
            "Interrupt function signature should not contain any lifetimes!";
            help = "Remove all function generics!"
        )
    });
    fn_sig.generics.type_params().for_each(|type_param| {
        emit_error!(
            type_param.span(),
            "Interrupt function signature should not contain any generics!";
            help = "Remove all function generics!"
        )
    });
    fn_sig.generics.const_params().for_each(|const_param| {
        emit_error!(
            const_param.span(),
            "Interrupt function signature should not contain any generics!";
            help = "Remove all function generics!"
        )
    });

    if let Some(abi) = fn_sig.abi.as_ref() {
        emit_error!(
            abi.span(),
            "Interrupt function signature should be of the ABI \"Rust\""
        )
    }

    match fn_sig.output {
        syn::ReturnType::Default => (),
        syn::ReturnType::Type(_, _) => {
            emit_error!(
                fn_sig.output.span(),
                "Interrupt function signature should only return ()";
                help = "Remove return type from function signature";
            );
        }
    }

    let mut inputs = fn_sig.inputs.iter();

    match inputs.next() {
        Some(FnArg::Typed(_)) => (),
        Some(_) | None => {
            emit_error!(
                fn_sig.inputs.span(),
                "Invalid Interrupt function signature!";
                help = "Expected Interrupt to accept argument of type 'arch::idt64::InterruptInfo'";
            );
        }
    }

    match inputs.next() {
        Some(arg) => {
            emit_error!(
                arg.span(),
                "Interrupt function signature should accept 0 or 1 arguments";
                help = "Remove excess arguments";
            );
        }
        _ => (),
    }
}
