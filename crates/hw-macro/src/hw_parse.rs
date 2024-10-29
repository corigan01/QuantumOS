use syn::{
    parse::Parse,
    token::{Mod, Struct},
    visit::{self, Visit},
    Attribute, Expr, ExprRange, Ident, ItemFn, ItemMod, Lit, LitInt, LitStr, PatLit, PatRange,
    Path, Token, Type, Visibility,
};

pub struct HwDeviceMacro {
    providers: Vec<MacroProviders>,
    fields: Vec<MacroFields>,
}

impl Parse for HwDeviceMacro {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut providers = Vec::new();
        let mut fields = Vec::new();

        loop {
            let lookahead = input.lookahead1();

            if lookahead.peek(Mod) {
                let module: MacroProviders = input.parse()?;
                println!("Mod : {}", module.module.ident);
                providers.push(module);
            } else if lookahead.peek(Struct) {
                println!("{:?}", input);
                todo!()
            } else if lookahead.peek(Token![#]) {
                let field: MacroFields = input.parse()?;
                println!("Field : {:#?}", field);
                fields.push(field);
            } else {
                break;
            }
        }

        Ok(HwDeviceMacro { providers, fields })
    }
}

#[derive(Debug)]
enum Access {
    RW,
    RO,
    WO,
    RW1C,
    RW1O,
}

mod access {
    syn::custom_keyword!(RW);
    syn::custom_keyword!(RO);
    syn::custom_keyword!(WO);
    syn::custom_keyword!(RW1C);
    syn::custom_keyword!(RW1O);
}

impl Parse for Access {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(access::RW) {
            input.parse::<access::RW>()?;
            Ok(Access::RW)
        } else if lookahead.peek(access::RO) {
            input.parse::<access::RO>()?;
            Ok(Access::RO)
        } else if lookahead.peek(access::WO) {
            input.parse::<access::WO>()?;
            Ok(Access::WO)
        } else if lookahead.peek(access::RW1C) {
            input.parse::<access::RW1C>()?;
            Ok(Access::RW1C)
        } else if lookahead.peek(access::RW1O) {
            input.parse::<access::RW1O>()?;
            Ok(Access::RW1O)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
pub enum Bits {
    Single(LitInt),
    Range(ExprRange),
}

impl Parse for Bits {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr: Expr = input.parse()?;
        match expr {
            Expr::Lit(syn::ExprLit {
                lit: Lit::Int(int), ..
            }) => Ok(Self::Single(int)),
            Expr::Range(range) => Ok(Self::Range(range)),
            _ => Err(input.error("Expected a bit (literal) or bit-range (eg. 1..2 or 5..=10)")),
        }
    }
}

#[derive(Debug)]
pub struct FieldArguments {
    access: Access,
    bits: Bits,
    parent: Option<Path>,
}

impl Parse for FieldArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let access = input.parse()?;
        input.parse::<Token![,]>()?;
        let bits = input.parse()?;

        let mut parent = None;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            parent = Some(input.parse()?);
        }

        Ok(Self {
            access,
            bits,
            parent,
        })
    }
}

#[derive(Debug)]
pub struct MacroFields {
    docs: Vec<Lit>,
    other_attr: Vec<Attribute>,
    args: FieldArguments,
    ident: Ident,
    vis: Visibility,
}

impl Parse for MacroFields {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrbues = input.call(Attribute::parse_outer)?;

        let mut other_attr = Vec::new();
        let mut docs = Vec::new();

        let mut field_attr = None;

        for attr in &attrbues {
            // Don't save ourself
            if attr.path().is_ident("field") {
                if field_attr.is_some() {
                    return Err(input.error("Cannot have multiple 'field' attributes per def."));
                }

                field_attr = Some(attr);
                continue;
            }

            if attr.path().is_ident("doc") {
                let Expr::Lit(syn::ExprLit { lit, .. }) = &attr.meta.require_name_value()?.value
                else {
                    return Err(input.error("Expected doc string"));
                };

                docs.push(lit.clone());
            } else {
                other_attr.push(attr.clone());
            }
        }

        let field_attr =
            field_attr.ok_or(input.error("Require a #[bit] attribute, but none found!"))?;

        let args = field_attr.parse_args()?;

        let vis = input.parse()?;
        let ident = input.parse()?;
        input.parse::<Token![,]>()?;

        Ok(Self {
            docs,
            other_attr,
            args,
            ident,
            vis,
        })
    }
}

pub struct MacroProviders {
    module: ItemMod,
    read_type: Option<Type>,
    write_type: Option<Type>,
    fn_def: FnReturnTypeVisitor,
}

impl Parse for MacroProviders {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let module: ItemMod = input.parse()?;

        let mut fn_def = FnReturnTypeVisitor::empty();
        fn_def.visit_item_mod(&module);

        Ok(Self {
            module,
            read_type: None,
            write_type: None,
            fn_def,
        })
    }
}

#[derive(Debug)]
struct FnReturnTypeVisitor {
    write_fn: Option<ItemFn>,
    read_fn: Option<ItemFn>,
}

impl FnReturnTypeVisitor {
    pub fn empty() -> Self {
        Self {
            write_fn: None,
            read_fn: None,
        }
    }
}

impl<'ast> Visit<'ast> for FnReturnTypeVisitor {
    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        let function_ident = &i.sig.ident;

        if function_ident == "read" {
            self.read_fn = Some(i.clone());
        } else if function_ident == "write" {
            self.write_fn = Some(i.clone());
        }

        visit::visit_item_fn(self, i);
    }
}
