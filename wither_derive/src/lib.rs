//! Wither's custom derive module.

#![recursion_limit = "200"]

mod model;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::DeriveInput;

use model::MetaModel;

/// Please see the wither crate's documentation for details on the Model derive system.
#[proc_macro_error]
#[proc_macro_derive(Model, attributes(model))]
pub fn proc_macro_derive_model(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(input as DeriveInput);
    let model = MetaModel::new(&input);
    model.expand().into()
}

// NOTE WELL: this is pending removed per https://github.com/thedodd/wither/issues/52
// /// Please see the wither crate's documentation for details on the Model derive system.
// #[proc_macro_error]
// #[proc_macro_derive(ModelSync, attributes(model))]
// pub fn proc_macro_derive_model_sync(input: TokenStream) -> TokenStream {
//     let input: DeriveInput = syn::parse_macro_input!(input as DeriveInput);
//     let model = MetaModel::new(&input);
//     model.expand_sync().into()
// }

#[cfg(test)]
mod test {
    #[test]
    fn derive_tests() {
        let t = trybuild::TestCases::new();
        t.pass("tests/pass/*.rs");
        t.compile_fail("tests/fail/*.rs");
    }
}
