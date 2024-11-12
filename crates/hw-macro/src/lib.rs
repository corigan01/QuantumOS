#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use provider_parse::MacroStruct;
use syn::parse_macro_input;

pub(crate) mod hw_gen;
pub(crate) mod hw_parse;

mod macro_gen;
pub(crate) mod make_hw_parse;
pub(crate) mod provider_parse;

#[proc_macro]
pub fn hw_device(tokens: TokenStream) -> TokenStream {
    let macro_input = parse_macro_input!(tokens as hw_parse::HwDeviceMacro);

    hw_gen::gen(macro_input).into()
}

#[proc_macro_attribute]
pub fn make_hw(args: TokenStream, input: TokenStream) -> TokenStream {
    let macro_fields = parse_macro_input!(args as make_hw_parse::MakeHwMacroInput);
    let struct_inner = parse_macro_input!(input as syn::ItemStruct);

    let macro_struct = MacroStruct {
        struct_inner,
        macro_fields,
    };

    macro_gen::gen_struct(macro_struct).into()
}

/*

Hmm I am not sure I am happy with the macro as it is right now.

What if we do something like this:

#[make_hw(
    /// Protected Mode Thing
    field(RO, 0, protected_mode),
    /// Dingus
    field(RW, 1..2, dingus_mode),
)]
mod cr0 {
    pub fn read() -> u32 {

    }

    pub fn write(input: u32) {

    }
}

const PORT0: AnyThing = 0;
const PORT1: AnyThing = 1;

#[make_hw(
    /// Protected Mode Thing
    field(RO, 0, protected_mode),
    /// Dingus
    field(RW, 1..2, dingus_mode),
    ports(PORT0, PORT1)
)]
mod dingus {
    pub fn read()
}


#[make_hw(
    /// Protected Mode Thing
    field(RO, 0, protected_mode),
    /// Dingus
    field(RW, 1..2, dingus_mode),
)]
struct DingusRegister {
    stuff: String,
    other_stuff: Vec<u32>,
    // This is the field that HW will use to read/write into
    //
    // Internally will just generate 'read() -> T' and 'write(val: T)'
    // for this field.
    //
    // If you don't use this, then you need to provide a 'read'
    // and 'write' function yourself.
    #[here_hw(read, write)]
    field: u32
}

impl DingusRegister {
    pub fn read(&self) -> u32 {
        self.field
    }

    pub fn write(&mut self, value: u32) {
        self.field = value;
    }
}


*/
