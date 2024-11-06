use proc_macro2::TokenStream;
use quote::quote;
use syn::Visibility;

use crate::{make_hw_parse::MakeHwMacroInput, provider_parse::MacroStruct};

enum SelfConditions {
    NoSelf,
    RefSelf,
    MutSelf,
}

struct FnGenConditions {
    inner_const: bool,
    inner_self: SelfConditions,
    vis: Visibility,
}

struct Fields<'a> {
    fields: &'a MakeHwMacroInput,
    read_fn: Option<FnGenConditions>,
    write_fn: Option<FnGenConditions>,
}

pub fn gen_struct(macro_struct: MacroStruct) -> TokenStream {
    Fields {
        fields: &macro_struct.macro_fields,
        read_fn: None,
        write_fn: None,
    };
    quote! {}
}
