//! Derive the `Model` trait on your data model structs.
//!
//! All `Model` struct & field attributes are declared inside of `model(...)` attributes as such: `#[model(<attrs>)]`.
//!
//! ### Derive
//! Deriving `Model` for your struct is straightforward.
//!
//! - Ensure that your struct has at least the following derivations: `#[derive(Model, Serialize, Deserialize)]`.
//! - Ensure that you have a field named `id`, of type `Option<bson::oid::ObjectId>`, with the following serde attributes: `#[serde(rename="_id", skip_serializing_if="Option::is_none")]`.
//!
//! For now, it seems logical to disallow customization of the PK. An argument could be made for allowing full customization of the PK for a MongoDB collection, but there really is no end-all reasoning for this argument which I am aware of. If you need to treat a different field as PK, then just add the needed index to the field, and you are good to go. More on indexing soon.
//!
//! If you need to implement `Serialize` and/or `Deserialize` manually, add the `#[model(skip_serde_checks)]` struct attribute, then you may remove the respective derivations mentioned above. If you are handling the `id` field manually as well, then you may remove the `rename` & `skip_serializing_if` attributes as well. However, take care to ensure that you are replicating the serde behavior of these two attributes, else you may run into strange behavior ... and it won't be my fault `;p`.
//!
//! ### Struct Attributes
//! There are a few struct-level `Model` attributes available currently.
//!
//! - `collection_name="..."`: this allows you to specify your model name explicitly. By default, your model's name will be pluralized, and then formatted as a standard table name.
//! - `skip_serde_checks`: including this attribute will disable any checks which are normally performed to ensure that serde is setup properly on your model. If you disable serde checks, you're on your own `:)`.
//!
//! ### Indexing
//! Adding indexes to your model's collection is done entirely through the attribute system. Let's start with an example.
//!
//! ```rust
//! An example model.
//! #[derive(Model, Serialize, Deserialize)]
//! struct MyModel {
//!     /// The ID of the model.
//!     #[serde(rename="_id", skip_serializing_if="Option::is_none")]
//!     pub id: Option<bson::oid::ObjectId>,
//!
//!     //! This field has a unique index on it.
//!     #[model(index(index_type="dsc", unique="true"))]
//!     pub email: String,
//!
//!     //! First field of a compound index.
//!     #[model(index(index_type="dsc", with(last_name="dsc")))]
//!     pub first_name: String,
//!
//!     //! Is indexed along with `first_name`, but nothing special is declared here.
//!     pub last_name: String,
//!
//!     // snip ...
//! }
//! ```
//!
//! As you can see, everything is declared within the `#[model(index(...))]` attributes. Let's break this down.
//!
//! ##### index
//! Everything related to an index declaration must be declared within these parens.
//!
//! ##### type
//! This declares the type of index for the field which this attribute appears on, which will also be the first field of the generated index. The value must be one of the valid MongoDB index types:  `"asc"`, `"dsc"`, `"2d"`, `"2dsphere"`, `"geoHaystack"`, `"text"` & `"hashed"`.
//!
//! ##### with
//! This is optional. For compound indexes, this is where you declare the other fields which the generated index is to be created with. Inside of these parens, you map field names to index types. The field name must be the name of the target field as it will be in the MongoDB collection. The value must be one of the valid MongoDB index types, as described above.
//!
//! ##### weights
//! This is optional. Values here simply map field names to `i32` values wrapped in strings.
//!
//! ```rust
//!     // snip ...
//!
//!     // A text search field, so we add a `weights` field on our index for optimization.
//!     #[model(index(index_type="text", with(text1="text"), weights(text0="10", text1="5")))]
//!     pub text0: String,
//!
//!     // The other field of our text index. No `model` attributes need to be added here.
//!     pub text1: String,
//!
//!     // snip ...
//! }
//! ```
//!
//! Check out the MongoDB docs on [Control Search Results with Weights](https://docs.mongodb.com/manual/tutorial/control-results-of-text-search/) for some excellent guidance on how to effectively use text indexes.
//!
//! ##### other attributes
//! Other attributes, like `unique` or `sparse`, are optional. Simply use the name of the attribute, followed by `=`, followed by the desired value (which must be quoted). Be sure to comma-separate each attribute-value pair. All attributes supported by the underlying MongoDB driver are supported by this framework. A list of all attributes can be found in the docs for [IndexOptions](https://docs.rs/mongodb/latest/mongodb/coll/options/struct.IndexOptions.html).
//!
//! ##### known issues
//! - As of the `0.6.0` implementation, specifying the additional fields of a compound index using this system my be theoretically limiting. Technically, the field names are declared as Rust `syn::Ident`s, which carries the restriction of being a valid variable name, which is more limiting than that of MongoDB's field naming restrictions. **Please open an issue** if you find this to be limiting. There are workarounds, but if this is a big issue, I definitely want to know. There are other ways this could be implemented.
//! - Indexing subdocuments is in progress, but not done yet. Will probably come as `0.7` or something.
//! - To index a field on a subdocument which is not modelled (EG, using `Document` as a value for a field), you will have to manually implement `Model` for your struct & then manually specify the indexes. See the docs in the `wither` crate for more information on manually implementing `Model` on your structs.
//!
//! ### Migrations
//! Migrations are not currently part of the `Model` derivation system. A separate pattern is used for migrations. Check out the docs on migrations in the `wither` crate.

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

#[proc_macro_derive(Model, attributes(model))]
pub fn proc_macro_derive_model(input: TokenStream) -> TokenStream {
    // Parse the input token stream into a syntax tree.
    let input: DeriveInput = syn::parse(input).expect("Unable to parse code for deriving `Model`.");

    // Build a meta model of the struct which `Model` is being derived on.
    let model = MetaModel::new(input);

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
