extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use syn::DeriveInput;

/// All `model` attributes which are available on the struct level.
struct ModelStructAttrs {
    /// The name to be used for the model's collection.
    ///
    /// TODO: maybe make this optional and just pluralize the struct ident as the name.
    collection_name: String,
}

struct ModelFieldAttrs {

}

#[proc_macro_derive(Model)]
pub fn proc_macro_derive_model(input: TokenStream) -> TokenStream {
    // Parse the input token stream into a syntax tree.
    let input: DeriveInput = syn::parse(input).expect("Unable to parse code for deriving `Model`.");

    // Extract struct data. We only support model derivation on structs.
    let struct_data = match input.data {
        syn::Data::Struct(struct_data) => struct_data,
        _ => panic!("Deriving `Model` is only supported on structs."),
    };

    // TODO: validate the anatomy of the derivation target.
    // Ensure the target struct has an elected `id` field with the needed type.
    must_have_id_field(&struct_data);

    // Get the name of the target struct.
    let name = input.ident;

    // TODO: ensure `derive(Serialize, Deserialize)` on the struct.
    // >>>

    // Collect model's struct & field attributes.
    let struct_attrs = collect_model_struct_attrs(input.attrs);
    let field_attrs = collect_model_field_attrs(&struct_data);

    // Build output code for deriving `Model`.
    let collection_name = struct_attrs.collection_name.as_str();
    let expanded = quote! {
        impl Model for #name {
            const COLLECTION_NAME: &str = #collection_name;

            /// Get a cloned copy of this instance's ID.
            fn id(&self) -> Option<bson::oid::ObjectId> {
                self.id.clone()
            }

            /// Set this instance's ID.
            fn set_id(&mut self, oid: bson::oid::ObjectId) {
                self.id = Some(oid);
            }
        }
    };

    // Send code back to compiler.
    expanded.into()
}

/// Collect all `model` attributes on the target struct.
fn collect_model_struct_attrs(attrs: Vec<syn::Attribute>) -> ModelStructAttrs {
    ModelStructAttrs{collection_name: "".to_owned()}
}

fn collect_model_field_attrs(target: &syn::DataStruct) -> bool {
    true
}

fn must_have_id_field(target: &syn::DataStruct) {

}
