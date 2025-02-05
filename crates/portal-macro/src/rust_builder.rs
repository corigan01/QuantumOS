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
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use quote::TokenStreamExt;
use quote::quote;

pub fn generate_rust_portal(portal: &ast::PortalMacro) -> TokenStream2 {
    quote! {
        #portal
    }
}

impl ToTokens for ast::PortalMacro {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let doc_attr = self.doc_attributes.iter();
        let portal_ident = self.get_mod_ident();
        let vis = &self.vis;

        tokens.append_all(quote! {
            #(#doc_attr)*
            #vis mod #portal_ident {

            }
        });
    }
}
