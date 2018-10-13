#![recursion_limit="200"]

#[macro_use]
extern crate bson;
extern crate inflector;
extern crate mongodb;
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate serde;
extern crate syn;

mod model;
mod model_field;
mod model_struct;
mod msg;
mod tokens;

use proc_macro::TokenStream;
use syn::DeriveInput;

use model::MetaModel;
use tokens::Indexes;

/// Derive the `Model` trait on your data model structs.
///
/// All `Model` struct & field attributes are declared inside of the `model(...)` ident
/// as such: `#[model(<attr>)]`.
///
/// ### Derive
/// Deriving `Model` for your struct is straightforward.
///
/// - Ensure that your struct has at least the following derivations: `#[derive(Model, Serialize, Deserialize)]`.
/// - Ensure that you have a field named `id`, of type `Option<bson::oid::ObjectId>`, with the
///   following serde attributes: `#[serde(rename="_id", skip_serializing_if="Option::is_none")]`.
///
/// For now, it seems logical to disallow customization of the PK. An argument could be made for
/// allowing full customization of the PK for a MongoDB collection, but there really is no end-all
/// reasoning for this argument which I am aware of. If you need to treat a different field as
/// PK, then just add the needed index to the field, and you are good to go. More on indexing soon.
///
/// If you need to implement `Serialize` and/or `Deserialize` manually, add the `#[model(skip_serde_checks=true)]`,
/// then you may remove the respective derivations mentioned above. If you are handling the `id`
/// field manually as well, then you may remove the `rename` & `skip_serializing_if` attributes as
/// well. However, take care to ensure that you are replicating the serde behavior of these two
/// attributes, else you may run into strange behavior ... and it won't be my fault `;p`.
///
/// ### Struct Attributes
/// There are a few struct-level `Model` attributes available currently.
///
/// - `collection_name` takes a literal string. This allows to specify your model name explicitly.
///   By default, your model's name will be pluralized, and then formatted as a standard table name.
/// - `skip_serde_checks` takes a literal boolean. Setting this to `true` will disable any checks
///   which are normally performed to ensure that serde is setup properly on your model. If you
///   disable serde checks, you're on your own `:)`.
///
/// ### Indexing
/// Adding indexes to your model's collection is done entirely through the attribute system. Let's
/// take a look.
///
/// Start off by adding one or more index declaration attributes to the field which is to be the
/// first field of the index: `#[model(<index declaration>)]`. The `<index declaration>`, must look
/// like this: `index(direction="...", with(...), (idx_attr="value",)*)`. Let's break this down.
///
/// - `index(...)` everything related to an index declaration must be declared within these parens.
/// - `direction="..."` declares the direction of the field which this attribute appears on, which
///   will also be the first field of the generated index. The value must be one of `"asc"` or
///   `"dsc"`. For a simple single-field index, this is all you need.
/// - `with(...)` is optional. For compound indexes, this is where you declare the other fields
///   which the generated index is to be created with. Inside of these parens, you map field names
///   to directions. The field name must be the name of the target field as it will be in the
///   MongoDB collection. The value must be either `"asc"` or `"dsc"`, as usual.
/// - `(index_attribute="value",)*` is also optional. This is where you specify the attributes of
///   the index itself. Simply use the name of the attribute, followed by `=`, followed by the
///   desired value (which must be quoted). Be sure to comma-separate each attribute-value pair.
///   All attributes supported by the underlying MongoDB driver are supported by this framework. A
///   list of all attributes can be found in the docs for [IndexOptions](https://docs.rs/mongodb/0.3.7/mongodb/coll/options/struct.IndexOptions.html).
///
/// ### Migrations
/// Migrations are not currently part of the `Model` derivation system. A separate pattern is used
/// for migrations. Check out the docs on migrations in the `wither` crate.
#[proc_macro_derive(Model, attributes(model))]
pub fn proc_macro_derive_model(input: TokenStream) -> TokenStream {
    // Parse the input token stream into a syntax tree.
    let input: DeriveInput = syn::parse(input).expect("Unable to parse code for deriving `Model`.");

    // Build a meta model of the struct which `Model` is being derived on.
    let model = MetaModel::new(input);

    // // TODO: >>>
    // // Ensure that target struct has needed serde derivations.
    // // meta.ensure_serde_derivations();

    // Ensure the target struct has `id` field with the needed attrs.
    model.ensure_id_field();

    // Build output code for deriving `Model`.
    let name = model.struct_name();
    let collection_name = model.collection_name();
    let indexes = Indexes(model.indexes());
    let expanded = quote! {
        impl<'a> wither::Model<'a> for #name {
            const COLLECTION_NAME: &'static str = #collection_name;

            /// Get a cloned copy of this instance's ID.
            fn id(&self) -> ::std::option::Option<::bson::oid::ObjectId> {
                self.id.clone()
            }

            /// Set this instance's ID.
            fn set_id(&mut self, oid: ::bson::oid::ObjectId) {
                self.id = Some(oid);
            }

            /// All indexes currently on this model.
            fn indexes() -> Vec<IndexModel> {
                #indexes
            }
        }
    };

    // Send code back to compiler.
    expanded.into()
}
