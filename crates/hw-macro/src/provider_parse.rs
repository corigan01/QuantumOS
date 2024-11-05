use syn::ItemStruct;

use crate::make_hw_parse::MakeHwMacroInput;

#[derive(Debug)]
pub struct MacroStruct {
    pub(crate) struct_inner: ItemStruct,
    pub(crate) macro_fields: MakeHwMacroInput,
}
