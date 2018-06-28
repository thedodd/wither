#[macro_use]
extern crate bson;
extern crate mongodb;
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use syn::DeriveInput;

use bson::Document;
use mongodb::coll::options::{IndexModel, IndexOptions};

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
    let meta = MetaModel::new(input);

    // // TODO: >>>
    // // Ensure that target struct has needed serde derivations.
    // // meta.ensure_serde_derivations();

    // Ensure the target struct has `id` field with the needed attrs.
    meta.ensure_id_field();

    // Build output code for deriving `Model`.
    let name = meta.struct_name();
    let collection_name = meta.collection_name();
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
        }
    };

    // Send code back to compiler.
    expanded.into()
}

//////////////////////////////////////////////////////////////////////////////
// Private Symbols ///////////////////////////////////////////////////////////

/// A meta representation of the `Model` derivation target.
struct MetaModel {
    ident: syn::Ident,
    struct_fields: syn::FieldsNamed,
    struct_data: MetaModelStructData,
    field_data: MetaModelFieldData,
}

impl MetaModel {
    /// Create a new instance.
    pub fn new(input: DeriveInput) -> Self {
        // The target's ident.
        let ident = input.ident;

        // Extract struct data. We only support model derivation on structs.
        let struct_fields = match input.data {
            syn::Data::Struct(struct_data) => match struct_data.fields {
                syn::Fields::Named(named_fields) => named_fields,
                _ => panic!("Structs used as a `Model` must have named fields."),
            },
            _ => panic!("Deriving `Model` is only supported on structs."),
        };

        // Extract struct & field attrs.
        let struct_data = MetaModelStructData::new(input.attrs.as_slice(), &ident);
        let field_data = MetaModelFieldData::new(&struct_fields);

        MetaModel{ident, struct_fields, struct_data, field_data}
    }

    /// The target struct's ident.
    pub fn struct_name(&self) -> &syn::Ident {
        &self.ident
    }

    /// The collection name to be used for this model.
    pub fn collection_name(&self) -> &str {
        self.struct_data.collection_name.as_str()
    }

    /// Ensure that the target struct has the needed `id` field.
    pub fn ensure_id_field(&self) {
        // Iterate over fields so that we can check for the `id` field & validate its attrs.
        for field in self.struct_fields.named.iter() {
            let ident = match field.ident {
                Some(ref ident) => ident.to_string(),
                _ => continue,
            };

            // We are only looking for the `id` field.
            if "id" != &ident { continue }

            // If serde checks have been disabled, then we are done here.
            if self.struct_data.skip_serde_checks {
                return;
            }

            // Check the `id` field's attrs.
            for attr in field.attrs.iter() {
                let meta = match attr.interpret_meta() {
                    Some(meta) => meta,
                    None => continue,
                };
                if meta.name() != "serde" {
                    continue;
                }
                let id_attrs = match meta {
                    syn::Meta::List(ref list) => list.nested.iter().by_ref()
                        .filter_map(|nested_meta| match nested_meta {
                            syn::NestedMeta::Meta(meta) => Some(meta),
                            _ => None,
                        }).filter_map(|meta| match meta {
                            syn::Meta::NameValue(kv) => Some(kv),
                            _ => None,
                        }).fold(NeededIdFieldSerdeAttrs::default(), |mut acc, kv| {
                            let val = match &kv.lit {
                                syn::Lit::Str(ref lit) => lit.value(),
                                _ => return acc,
                            };
                            match kv.ident.to_string().as_str() {
                                "rename" => {
                                    acc.rename = val;
                                },
                                "skip_serializing_if" => {
                                    acc.skip_serializing_if = val;
                                }
                                _ => ()
                            };
                            acc
                        }),
                    _ => continue,
                };

                // Ensure needed ID attrs are present & return from function if we are g2g.
                id_attrs.validate();
                return;
            }
            panic!(ID_FIELD_SERDE_REQ_MSG);
        }
        panic!(ID_FIELD_SERDE_REQ_MSG);
    }
}

/// All `Model` struct attributes which have been accumulated from the target struct.
#[derive(Default)]
struct MetaModelStructData {
    /// The name to be used for the model's collection.
    pub collection_name: String,

    /// An attribute to control whether or not this derive macro will check for serde attributes.
    pub skip_serde_checks: bool,
}

impl MetaModelStructData {
    /// Extract needed data from the target model's struct attributes.
    pub fn new(attrs: &[syn::Attribute], target_ident: &syn::Ident) -> Self {
        // Collect the target's struct level `model` attrs.
        let mut data = attrs.iter().fold(MetaModelStructData::default(), |mut acc, attr| {
            // Ensure attr is structured properly.
            let meta = match attr.interpret_meta() {
                Some(meta) => meta,
                None => return acc,
            };

            // If we are not looking at a `model` attr, then skip.
            if meta.name() != "model" {
                return acc;
            }

            // Unpack the inner attr's components.
            match meta {
                syn::Meta::List(list) => list.nested.iter().by_ref()
                    .filter_map(|nested_meta| match nested_meta {
                        syn::NestedMeta::Meta(meta) => Some(meta),
                        _ => panic!(MODEL_ATTR_FORM_MSG),
                    }).filter_map(|meta| match meta {
                        syn::Meta::NameValue(kv) => Some(kv),
                        _ => panic!(MODEL_STRUCT_ATTR_FORM_MSG),
                    }).for_each(|kv| {
                        let ident = kv.ident.to_string();
                        match &kv.lit {
                            syn::Lit::Str(ref val) => {
                                let value = val.value();
                                match ident.as_str() {
                                    "collection_name" => {
                                        acc.collection_name = value;
                                        if acc.collection_name.len() < 1 {
                                            panic!("The `Model` struct attribute 'collection_name' may not have a zero-length value.");
                                        }
                                    },
                                    "skip_serde_checks" => {
                                        acc.skip_serde_checks = value.parse().expect("Value for `skip_serde_checks` must be parseable as a `bool`.");
                                    },
                                    _ => panic!(format!("Unrecognized struct-level `Model` attribute '{}'.", ident)),
                                }
                            },
                            _ => panic!("Only string literals are supported as named values in `Model` attributes."),
                        }
                    }),
                _ => panic!(MODEL_ATTR_FORM_MSG),
            };
            acc
        });

        // If collection name is default "", then use the struct's ident.
        // TODO: PLURALIZE & FORMAT AS TABLE NAME.
        data.collection_name = target_ident.to_string();
        data
    }
}

/// The `Model` field attributes which have been accumulated from the target model.
#[derive(Default)]
struct MetaModelFieldData;

impl MetaModelFieldData {
    /// Extract needed data from the target model's field attributes.
    pub fn new(struct_fields: &syn::FieldsNamed) -> Self {
        // Iterate over fields of the target model & accumulate needed data.
        struct_fields.named.iter()
            // Filter out fields with no attrs.
            .filter_map(|field| {
                if field.attrs.len() == 0 { None } else { Some(field) }
            })
            // Filter out fields which do not have `model` attrs, and group as `(Field, Vec<Vec<Meta>>)`.
            .fold(MetaModelFieldData::default(), |mut acc, field| {
                for attr in field.attrs.iter() {
                    let meta = match attr.interpret_meta() {
                        Some(meta) => meta,
                        None => continue,
                    };
                    if meta.name() != "model" {
                        continue;
                    }

                    // Collect `model` elements as vector of meta objects.
                    let meta_list = match meta {
                        syn::Meta::List(meta_list) => meta_list,
                        _ => panic!(MODEL_ATTR_FORM_MSG),
                    };
                    let inner_meta = meta_list.nested.iter().map(|nested| {
                        match nested {
                            syn::NestedMeta::Meta(meta) => meta,
                            _ => panic!(MODEL_ATTR_FORM_MSG),
                        }
                    }).collect::<Vec<&syn::Meta>>();

                    // Iterate over the inner meta elements & handle as needed.
                    for inner in inner_meta {
                        let attr_name = inner.name().to_string();
                        match attr_name.as_ref() {
                            // Handle index attrs.
                            "index" => {
                                let model = MetaModelFieldData::build_index_model(field, inner);
                            },
                            _ => panic!("Unrecognized `model` field attribute '{}'.", attr_name),
                        }
                    }
                }
                acc
            })
    }

    /// Build an `IndexModel` from the metadata in the given attr meta object.
    pub fn build_index_model(field: &syn::Field, index_container: &syn::Meta) {
        // Collect the internals of the `index` attr as a vector of additional meta elements.
        let index_attrs = match index_container {
            syn::Meta::List(meta_list) => meta_list.nested.iter().map(|nested| {
                match nested {
                    syn::NestedMeta::Meta(meta) => meta,
                    _ => panic!(MODEL_ATTR_INDEX_ELEMENT_FORM_MSG),
                }
            }).collect::<Vec<&syn::Meta>>(),
            _ => panic!(MODEL_ATTR_INDEX_FORM_MSG),
        };

        // A few symbols needed for building a new index model.
        let mut keys = doc!{};
        let mut opts = IndexOptions::new();
        let name = MetaModelFieldData::get_db_field_name(field);

        // Iterator over index metadata.
        for idx_elem in index_attrs {
            let elem_name = idx_elem.name().to_string();
            match elem_name.as_ref() {
                "direction" => MetaModelFieldData::index_direction(idx_elem, &mut keys, &name),
                // "with" => MetaModelFieldData::index_with(idx_elem, &mut keys),
                // "background" => MetaModelFieldData::index_background(idx_elem),
                // "expire_after_seconds" => MetaModelFieldData::index_expire_after_seconds(idx_elem),
                // "name" => MetaModelFieldData::index_name(idx_elem),
                // "sparse" => MetaModelFieldData::index_sparse(idx_elem),
                // "storage_engine" => MetaModelFieldData::index_storage_engine(idx_elem),
                // "unique" => MetaModelFieldData::index_unique(idx_elem),
                // "version" => MetaModelFieldData::index_version(idx_elem),
                // "default_language" => MetaModelFieldData::index_default_language(idx_elem),
                // "language_override" => MetaModelFieldData::index_language_override(idx_elem),
                // "text_version" => MetaModelFieldData::index_text_version(idx_elem),
                // "weights" => MetaModelFieldData::index_weights(idx_elem),
                // "sphere_version" => MetaModelFieldData::index_sphere_version(idx_elem),
                // "bits" => MetaModelFieldData::index_bits(idx_elem),
                // "max" => MetaModelFieldData::index_max(idx_elem),
                // "min" => MetaModelFieldData::index_min(idx_elem),
                // "bucket_size" => MetaModelFieldData::index_bucket_size(idx_elem),
                _ => panic!("Unrecognized `#[model(index(...))]` attribute '{}'.", elem_name),
            };
        }
        // TODO:>>>
        // IndexModel::new(keys, Some(opts))
    }

    /// Handle the `direction` index key.
    pub fn index_direction(attr: &syn::Meta, keys: &mut Document, field_name: &str) {
        let name_value = match attr {
            syn::Meta::NameValue(kv) => kv,
            _ => panic!("The index `#[model(index(direction))]` key must be a name-value pair."),
        };
        let lit_val = match name_value.lit {
            syn::Lit::Str(ref val) => val.value(),
            _ => panic!(MODEL_ATTR_INDEX_DIRECTION_ALLOWED_VALUES_MSG),
        };
        if lit_val != "asc" && lit_val != "dsc" {
            panic!(MODEL_ATTR_INDEX_DIRECTION_ALLOWED_VALUES_MSG);
        }

        // Everything checks out, so add the index direction for the target field.
        keys.insert(field_name, lit_val);
    }

    // pub fn index_with(attr: &syn::Meta, keys: &mut Document) {}

    // "with" => (),
    // "background" => (), // Option<bool>
    // "expire_after_seconds" => (), // Option<i32>
    // "name" => (), // Option<String>
    // "sparse" => (), // Option<bool>
    // "storage_engine" => (), // Option<String>
    // "unique" => (), // Option<bool>
    // "version" => (), // Option<i32>
    // "default_language" => (), // Option<String>
    // "language_override" => (), // Option<String>
    // "text_version" => (), // Option<i32>
    // "weights" => (), // Option<Document>
    // "sphere_version" => (), // Option<i32>
    // "bits" => (), // Option<i32>
    // "max" => (), // Option<f64>
    // "min" => (), // Option<f64>
    // "bucket_size" => (), // Option<i32>

    /// Get the name that is to be used for the target field inside of the MongoDB database.
    pub fn get_db_field_name(field: &syn::Field) -> String {
        // Check for a `serde(rename)` attr.
        for attr in field.attrs.iter() {
            let meta = match attr.interpret_meta() {
                Some(meta) => meta,
                None => continue,
            };
            if meta.name() != "serde" {
                continue;
            }
            let serde_name = match meta {
                syn::Meta::List(ref list) => {
                    let serde_name = list.nested.iter().by_ref()
                        .filter_map(|nested_meta| match nested_meta {
                            syn::NestedMeta::Meta(meta) => Some(meta),
                            _ => None,
                        }).filter_map(|meta| match meta {
                            syn::Meta::NameValue(kv) => Some(kv),
                            _ => None,
                        }).filter_map(|kv| {
                            if kv.ident != "rename" {
                                return None;
                            }
                            match kv.lit {
                                syn::Lit::Str(ref lit) => Some(lit.value()),
                                _ => None,
                            }
                        }).last();
                    match serde_name {
                        Some(name) => name,
                        _ => continue,
                    }
                },
                _ => continue,
            };
            return serde_name;
        }

        // Serde rename not found, so use field ident.
        field.ident.as_ref()
            .expect("Programming error. This unwrap should only ever be called on named model field idents. Please file a bug on github.")
            .to_string()
    }
}

/// The serde attributes which are needed on the ID field for the system to work correctly.
#[derive(Default)]
struct NeededIdFieldSerdeAttrs {
    rename: String,
    skip_serializing_if: String,
}

impl NeededIdFieldSerdeAttrs {
    /// Validate that the fields are holding the needed values.
    fn validate(&self) {
        if self.rename != "_id" {
            panic!(r#"A `Model`'s 'id' field must have a serde `rename` attribute with a value of `"_id"`."#)
        }
        if self.skip_serializing_if != "Option::is_none" {
            panic!(r#"A `Model`'s 'id' field must have a serde `skip_serializing_if` attribute with a value of `"Option::is_none"`."#)
        }
    }
}

////////////////////
// Error Messages //

/// An error message indicating the serde field attribute requirements on the `Model` `id` field.
const ID_FIELD_SERDE_REQ_MSG: &str = r#"A `Model` struct must have a field named `id`, and it must have the following attribute: `#[serde(rename="_id", skip_serializing_if="Option::is_none")]`."#;

/// An error message indicating the required form of `Model` attributes.
const MODEL_ATTR_FORM_MSG: &str = "All `Model` attributes must take the form `#[model(...)]`.";

/// An error message indicating the required form of a `#[model(index(...))]` attribute.
const MODEL_ATTR_INDEX_FORM_MSG: &str = "The `model(index)` attribute must have its own set of values, as such: `#[model(index(...))]`.";

/// An error message indicating the required form of elements within an index declaration.
const MODEL_ATTR_INDEX_ELEMENT_FORM_MSG: &str = "Index declarations on your model fields may only contain name-value pairs or the nested `with(...)` element.";

/// An error message indicating the allowed values for `#[model(index(direction))]`.
const MODEL_ATTR_INDEX_DIRECTION_ALLOWED_VALUES_MSG: &str = r#"The index `direction` value must be one of `"asc"` or `"dsc"`."#;

/// An error message indicating the required form of a `Model`'s struct level attributes.
const MODEL_STRUCT_ATTR_FORM_MSG: &str = r#"A `Model`'s struct level attributes may only contain name-value pairs: `#[model(name="value")]`."#;
