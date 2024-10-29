use syn::{
    parse::Parse,
    token::{Mod, Struct},
    visit::{self, Visit},
    ItemFn, ItemMod, Type,
};

pub struct HwDeviceMacro {
    providers: Vec<MacroProviders>,
}

impl Parse for HwDeviceMacro {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut providers = Vec::new();

        loop {
            let lookahead = input.lookahead1();
            if lookahead.peek(Mod) {
                let module: MacroProviders = input.parse()?;
                println!("Mod : {}", module.module.ident);
                providers.push(module);
            } else if lookahead.peek(Struct) {
                println!("{:?}", input);
                todo!()
            } else {
                break;
            }
        }

        Ok(HwDeviceMacro { providers })
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
