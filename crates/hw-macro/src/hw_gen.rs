use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Expr, Ident, Lit, Type};

use crate::hw_parse::{Access, Bits, HwDeviceMacro, MacroFields, MacroProviders};

#[derive(Debug)]
struct ProviderInfo<'a> {
    provider: &'a MacroProviders,

    readable_bits: Option<usize>,
    writeable_bits: Option<usize>,
    can_read_be_const: bool,
    can_write_be_const: bool,
    is_read_unsafe: bool,
    is_write_unsafe: bool,
}

impl<'a> ProviderInfo<'a> {
    fn convert_type_to_bits(bit_type: &Type) -> Option<usize> {
        let Type::Path(path) = bit_type else {
            return None;
        };

        match path.path.get_ident()?.to_string().as_str() {
            "u64" | "i64" => Some(64),
            "u32" | "i32" => Some(32),
            "u16" | "i16" => Some(16),
            "u8" | "i8" => Some(8),
            _ => {
                path.path
                    .get_ident()
                    .span()
                    .unwrap()
                    .error("Unsupported return type for function!")
                    .help("Use 'u64', 'u32', 'u16', 'u8', or their signed versions.")
                    .emit();
                None
            }
        }
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

        let is_read_unsafe = provider
            .fn_def
            .read_fn
            .as_ref()
            .is_some_and(|inner| inner.sig.unsafety.is_some());

        let is_write_unsafe = provider
            .fn_def
            .write_fn
            .as_ref()
            .is_some_and(|inner| inner.sig.unsafety.is_some());

        Self {
            provider,
            readable_bits,
            writeable_bits,
            can_read_be_const,
            can_write_be_const,
            is_read_unsafe,
            is_write_unsafe,
        }
    }

    fn _read_write_size_different(&self) -> bool {
        self.readable_bits
            .is_some_and(|read| self.writeable_bits.is_some_and(|write| write != read))
    }

    fn safe_rw_bits(&self) -> usize {
        self.readable_bits.min(self.writeable_bits).unwrap_or(0)
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

fn parse_range_piece(expr: &Expr, span: Span) -> usize {
    match expr {
        Expr::Lit(syn::ExprLit {
            lit: Lit::Int(int_lit),
            ..
        }) => int_lit
            .base10_parse()
            .map_err(|_| {
                span.unwrap()
                    .error("Cannot parse number, please provide a integer.")
                    .emit()
            })
            .unwrap_or(0),
        _ => {
            span.unwrap()
                .error("Only support literal values in range!")
                .emit();

            0
        }
    }
}

fn parse_range(range_expr: &syn::ExprRange, defaults: (usize, usize)) -> (usize, usize) {
    let start_bit = range_expr
        .start
        .as_ref()
        .map(|expr| parse_range_piece(&expr, range_expr.span()))
        .unwrap_or(defaults.0);

    let end_bit = range_expr
        .end
        .as_ref()
        .map(|expr| parse_range_piece(&expr, range_expr.span()))
        .unwrap_or(defaults.1);

    (start_bit, end_bit)
}

fn type_for_bits(bits: &Bits, max: usize) -> TokenStream {
    match bits {
        Bits::Single(_) => quote! {bool},
        Bits::Range(ref range_expr) => {
            let (start, mut end) = parse_range(range_expr, (0, max));

            if matches!(range_expr.limits, syn::RangeLimits::Closed(_)) {
                end += 1;
            }

            let n_bits = end - start;

            match n_bits {
                ..=8 => quote! { u8 },
                ..=16 => quote! { u16 },
                ..=32 => quote! { u32 },
                ..=64 => quote! { u64 },
                _ => quote! { () },
            }
        }
    }
}

fn gen_consts(provider_info: &ProviderInfo, field: &MacroFields) -> TokenStream {
    let safe_rw_range = provider_info.safe_rw_bits();
    let vis = &field.vis;

    let raw_module_name = provider_info.provider.module.ident.to_string();
    let (_, title_formatted_ident) = field.ident.to_string().chars().fold(
        (true, String::new()),
        |(mut new_word, mut acc), c| {
            match c {
                '_' => {
                    acc.push(' ');
                    new_word = true;
                }
                c if new_word => {
                    acc.push(c.to_ascii_uppercase());
                    new_word = false;
                }
                c => {
                    acc.push(c.to_ascii_lowercase());
                    new_word = false;
                }
            }

            (new_word, acc)
        },
    );

    let docs = &field.docs;

    match field.args.bits {
        Bits::Single(ref bit_literal) => {
            let Ok(bit_value): Result<usize, _> = bit_literal.base10_parse() else {
                bit_literal
                    .span()
                    .unwrap()
                    .error("Cannot parse literal")
                    .emit();

                return quote! {};
            };

            let const_offset = make_const_ident(&field.ident, "_OFFSET");

            quote_spanned! {field.span=>
                #[allow(unused)]
                #(#[doc = #docs])*
                #vis const #const_offset : usize = #bit_value;
            }
        }
        Bits::Range(ref range_expr) => {
            let (start, mut end) = parse_range(range_expr, (0, safe_rw_range - 1));

            if matches!(range_expr.limits, syn::RangeLimits::Closed(_)) {
                end += 1;
            }

            if start >= safe_rw_range || end >= safe_rw_range {
                range_expr
                    .span()
                    .unwrap()
                    .error(format!(
                        "Bits requested is larger than the max possible {}",
                        safe_rw_range
                    ))
                    .emit();

                return quote! {};
            }

            let const_mask = make_const_ident(&field.ident, "_MASK");
            let const_offset = make_const_ident(&field.ident, "_OFFSET");
            let const_max = make_const_ident(&field.ident, "_MAX_VALUE");
            let const_bits = make_const_ident(&field.ident, "_N_BITS");

            let value_bits = end - start;
            let value_max: u64 = 2u64.pow(value_bits as u32) - 1;
            let value_offset = start;
            let value_mask: u64 = value_max << value_offset;

            let doc_mask = format!(
                " Bit mask for '{}' in `{}`.",
                title_formatted_ident, raw_module_name
            );
            let doc_offset = format!(
                " Field offset for '{}' in `{}`.",
                title_formatted_ident, raw_module_name
            );
            let doc_max = format!(" Max possible value for '{}'.", title_formatted_ident);
            let doc_bits = format!(" Number of bits a '{}' can have.", title_formatted_ident);

            quote_spanned! {field.span=>
                #[doc = #doc_mask]
                #(#[doc = #docs])*
                #[allow(unused)]
                #vis const #const_mask : u64 = #value_mask;

                #[doc = #doc_max]
                #(#[doc = #docs])*
                #[allow(unused)]
                #vis const #const_max : u64 = #value_max;

                #[doc = #doc_offset]
                #(#[doc = #docs])*
                #[allow(unused)]
                #vis const #const_offset : usize = #value_offset;

                #[doc = #doc_bits]
                #(#[doc = #docs])*
                #[allow(unused)]
                #vis const #const_bits : usize = #value_bits;
            }
        }
    }
}

fn gen_read(field: &MacroFields, provider_info: &ProviderInfo, access: &Access) -> TokenStream {
    let is_multi = matches!(field.args.bits, Bits::Range(_));
    let vis = &field.vis;

    let (function_header, function_footer) = match access {
        Access::RW | Access::RO if is_multi => ("get_", ""),
        Access::RW | Access::RO if !is_multi => ("is_", "_set"),
        Access::RW1C => ("is_", "_clear"),
        Access::RW1O => ("is_", "_pending"),
        _ => ("", ""),
    };

    let function_ident = Ident::new(
        &format!("{}{}{}", function_header, &field.ident, function_footer),
        Span::call_site(),
    );

    let mut function_signature = Vec::new();

    if provider_info.can_read_be_const {
        function_signature.push(quote! { const });
    }

    if provider_info.is_read_unsafe {
        function_signature.push(quote! { unsafe })
    }

    let return_type = type_for_bits(&field.args.bits, provider_info.readable_bits.unwrap_or(0));
    let read_type = provider_info
        .provider
        .read_type
        .as_ref()
        .expect("Need read() function");

    function_signature.push(quote! { #vis fn #function_ident() -> #return_type });

    let provider_path = &provider_info.provider.module.ident;
    let read_val = quote! { #provider_path::read() };
    let offset_ident = make_const_ident(&field.ident, "_OFFSET");

    if is_multi {
        let mask_ident = make_const_ident(&field.ident, "_MASK");

        function_signature.push(quote! {{
           let inner: #read_type = #read_val;
           ((inner & (#mask_ident as #read_type)) >> (#offset_ident as #read_type)) as #return_type
        }});
    } else {
        function_signature.push(quote! {{
            let inner: #read_type = #read_val;
            (inner & (#offset_ident as #read_type)) != 0
        }});
    }

    let docs = &field.docs;
    let attr = &field.other_attr;

    quote_spanned! {field.span=>
        #(#[doc = #docs])*
        #(#attr)*
        #[allow(unused)]
        #(#function_signature)*
    }
}

fn gen_write(field: &MacroFields, provider_info: &ProviderInfo, access: &Access) -> TokenStream {
    let is_multi = matches!(field.args.bits, Bits::Range(_));
    let vis = &field.vis;

    let (function_header, function_footer) = match access {
        Access::RW | Access::RO if is_multi => ("write_", ""),
        Access::RW | Access::RO if !is_multi => ("set_", "_flag"),
        Access::RW1C => ("clear_", ""),
        Access::RW1O => ("activate_", ""),
        _ => ("", ""),
    };

    let function_ident = Ident::new(
        &format!("{}{}{}", function_header, &field.ident, function_footer),
        Span::call_site(),
    );

    let mut function_signature = Vec::new();

    // If both are const, we can also be const
    if provider_info.can_write_be_const && provider_info.can_read_be_const {
        function_signature.push(quote! { const });
    }

    // if either are unsafe, we are unsafe
    if provider_info.is_write_unsafe || provider_info.is_read_unsafe {
        function_signature.push(quote! { unsafe })
    }

    let arg_type = type_for_bits(&field.args.bits, provider_info.readable_bits.unwrap_or(0));
    let write_type = provider_info
        .provider
        .write_type
        .as_ref()
        .expect("Need write() function");

    function_signature.push(quote! { #vis fn #function_ident(value: #arg_type) });

    let provider_path = &provider_info.provider.module.ident;
    let write = quote! { #provider_path::write(val) };
    let read = quote! { #provider_path::read() };
    let offset_ident = make_const_ident(&field.ident, "_OFFSET");

    if is_multi {
        let mask_ident = make_const_ident(&field.ident, "_MASK");

        function_signature.push(quote! {{
            let inner = (#read as #write_type);
            let val = (inner & !(#mask_ident as #write_type)) | ((value as #write_type) << (#offset_ident as #write_type));
            #write;
        }});
    } else {
        function_signature.push(quote! {{
            let inner = (#read as #write_type);
            let val = (inner & !(#offset_ident as #write_type)) | ((if value { 1 } else { 0 }) << (#offset_ident as #write_type));
            #write;
        }});
    }

    let docs = &field.docs;
    let attr = &field.other_attr;

    quote_spanned! {field.span=>
        #(#[doc = #docs])*
        #(#attr)*
        #[allow(unused)]
        #(#function_signature)*
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

    let mut gen_tokens = Vec::new();
    gen_tokens.push(gen_consts(our_provider, &field));

    // Read
    match field.args.access {
        Access::WO => (),
        _ => gen_tokens.push(gen_read(field, our_provider, &field.args.access)),
    }

    // Write
    match field.args.access {
        Access::RO => (),
        _ => gen_tokens.push(gen_write(field, our_provider, &field.args.access)),
    }

    quote_spanned! {field.span=>
        #(#gen_tokens)*
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
