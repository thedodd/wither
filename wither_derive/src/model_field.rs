use std::fmt::Display;
use std::str::FromStr;

use mongodb::{
    coll::options::{IndexModel, IndexOptions},
    Document,
};
use syn;

use msg;

/// The `Model` field attributes which have been accumulated from the target model.
#[derive(Default)]
pub(crate) struct MetaModelFieldData {
    pub indexes: Vec<IndexModel>,
}

impl MetaModelFieldData {
    /// Extract needed data from the target model's field attributes.
    pub fn new(struct_fields: &syn::FieldsNamed) -> MetaModelFieldData {
        // Iterate over fields of the target model & accumulate needed data.
        struct_fields
            .named
            .iter()
            // Filter out fields with no attrs.
            .filter_map(|field| {
                if field.attrs.len() == 0 {
                    None
                } else {
                    Some(field)
                }
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
                        _ => panic!(msg::MODEL_ATTR_FORM),
                    };
                    let inner_meta = meta_list
                        .nested
                        .iter()
                        .map(|nested| match nested {
                            syn::NestedMeta::Meta(meta) => meta,
                            _ => panic!(msg::MODEL_ATTR_FORM),
                        })
                        .collect::<Vec<&syn::Meta>>();

                    // Iterate over the inner meta elements & handle as needed.
                    for inner in inner_meta {
                        let attr_name = inner.name().to_string();
                        match attr_name.as_ref() {
                            // Handle index attrs.
                            "index" => {
                                let model = build_index_model(field, inner);
                                acc.indexes.push(model);
                            }
                            _ => panic!("Unrecognized `model` field attribute '{}'.", attr_name),
                        }
                    }
                }
                acc
            })
    }
}

/// Build an `IndexModel` from the metadata in the given attr meta object.
fn build_index_model(field: &syn::Field, index_container: &syn::Meta) -> IndexModel {
    // Collect the internals of the `index` attr as a vector of additional meta elements.
    let index_attrs = match index_container {
        syn::Meta::List(meta_list) => meta_list
            .nested
            .iter()
            .map(|nested| {
                match nested {
                syn::NestedMeta::Meta(meta) => meta,
                _ => panic!("Literal string values are not allowed on their own within index declarations."), // Only other option is `Literal(Lit)`, as of 2018.11.09.
            }
            })
            .collect::<Vec<&syn::Meta>>(),
        _ => panic!(msg::MODEL_ATTR_INDEX_FORM),
    };

    // A few symbols needed for building a new index model.
    let mut index_root_model = IndexRoot::default();
    let mut opts = IndexOptions::new();
    index_root_model.field = get_db_field_name(field);

    // Iterate over index meta attrs and handle them as needed.
    for index_meta in index_attrs {
        let meta_elem_name = index_meta.name().to_string();
        match meta_elem_name.as_ref() {
            "index" => index_type(index_meta, &meta_elem_name, &mut index_root_model),
            "embedded" => index_embedded(index_meta, &mut index_root_model),
            "with" => index_with(index_meta, &mut index_root_model),
            "background" => opts.background = Some(extract_meta_kv(index_meta, &meta_elem_name)),
            "expire_after_seconds" => {
                opts.expire_after_seconds = Some(extract_meta_kv(index_meta, &meta_elem_name))
            }
            "name" => opts.name = Some(extract_meta_kv(index_meta, &meta_elem_name)),
            "sparse" => opts.sparse = Some(extract_meta_kv(index_meta, &meta_elem_name)),
            "storage_engine" => {
                opts.storage_engine = Some(extract_meta_kv(index_meta, &meta_elem_name))
            }
            "unique" => opts.unique = Some(extract_meta_kv(index_meta, &meta_elem_name)),
            "version" => opts.version = Some(extract_meta_kv(index_meta, &meta_elem_name)),
            "default_language" => {
                opts.default_language = Some(extract_meta_kv(index_meta, &meta_elem_name))
            }
            "language_override" => {
                opts.language_override = Some(extract_meta_kv(index_meta, &meta_elem_name))
            }
            "text_version" => {
                opts.text_version = Some(extract_meta_kv(index_meta, &meta_elem_name))
            }
            "weight" => index_weight(index_meta, &mut opts),
            "sphere_version" => {
                opts.sphere_version = Some(extract_meta_kv(index_meta, &meta_elem_name))
            }
            "bits" => opts.bits = Some(extract_meta_kv(index_meta, &meta_elem_name)),
            "max" => opts.max = Some(extract_meta_kv(index_meta, &meta_elem_name)),
            "min" => opts.min = Some(extract_meta_kv(index_meta, &meta_elem_name)),
            "bucket_size" => opts.bucket_size = Some(extract_meta_kv(index_meta, &meta_elem_name)),
            _ => panic!(
                "Unrecognized `#[model(index(...))]` attribute '{}'.",
                meta_elem_name
            ),
        };
    }
    IndexModel::new(index_root_model.final_keys(), Some(opts))
}

/// Handle the `embedded` index option.
///
/// Here we simply need to extract the value of the given `embedded` token, trim it a bit to ensure
/// that it is kosher, then add it to the IndexRoot model.
fn index_embedded(embedded: &syn::Meta, index_root: &mut IndexRoot) {
    let val = match embedded {
        syn::Meta::NameValue(kv) => match &kv.lit {
            syn::Lit::Str(litstr) => litstr.value().as_str().trim().trim_matches('.').to_string(),
            _ => panic!(msg::MODEL_ATTR_INDEX_EMBEDDED_FORM),
        },
        _ => panic!(msg::MODEL_ATTR_INDEX_EMBEDDED_FORM), // Other options would be `Word` or `List`.
    };
    index_root.embedded = Some(val);
}

/// Handle the `weight` index option.
fn index_weight(weight: &syn::Meta, opts: &mut IndexOptions) {
    // Collect internals of `index(weight(...))` attr as IndexWeight model.
    let index_weight_model = match weight {
        syn::Meta::List(meta_list) => {
            // Ensure the `weight(...)` declaration has the expected length of 2.
            if meta_list.nested.len() != 2 {
                panic!(msg::MODEL_ATTR_INDEX_WEIGHTS_FORM);
            }

            // Parse out contents of attr into IndexWeight data model.
            meta_list
                .nested
                .iter()
                .fold(IndexWeight::default(), |mut acc, nested| match nested {
                    syn::NestedMeta::Meta(meta) => match meta {
                        syn::Meta::NameValue(name_val) => match name_val.ident.to_string().as_str()
                        {
                            "field" => {
                                let val = extract_lit_str(
                                    &name_val.lit,
                                    msg::MODEL_ATTR_INDEX_WEIGHTS_FORM,
                                );
                                acc.field = val;
                                acc
                            }
                            "weight" => {
                                let val = extract_lit_str(
                                    &name_val.lit,
                                    msg::MODEL_ATTR_INDEX_WEIGHTS_FORM,
                                )
                                .parse::<i32>()
                                .expect(msg::MODEL_ATTR_INDEX_WEIGHTS_FORM);
                                acc.weight = val;
                                acc
                            }
                            _ => panic!(msg::MODEL_ATTR_INDEX_WEIGHTS_FORM),
                        },
                        _ => panic!(msg::MODEL_ATTR_INDEX_WEIGHTS_FORM),
                    },
                    _ => panic!(msg::MODEL_ATTR_INDEX_WEIGHTS_FORM),
                })
        }
        _ => panic!(msg::MODEL_ATTR_INDEX_WEIGHTS_FORM),
    };

    // Update the given index model's weights option.
    let opt_weights = opts.weights.get_or_insert_with(|| Document::new());
    opt_weights.insert(index_weight_model.field, index_weight_model.weight);
}

/// A generic value extraction method for extracting the value of a `syn::Meta::NameValue` field.
fn extract_meta_kv<T>(attr: &syn::Meta, attr_name: &str) -> T
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    let name_value = match attr {
        syn::Meta::NameValue(kv) => kv,
        _ => panic!(format!(
            "The index `#[model(index({}))]` key must be a name-value pair.",
            attr_name
        )),
    };
    match name_value.lit {
        syn::Lit::Str(ref val) => match val.value().parse() {
            Ok(val) => val,
            Err(err) => panic!(format!("Invalid type provided as value for the `#[model(index({}))]` attribute. Must be wrapped in quotes. {}", attr_name, err)),
        },
        _ => panic!("Named values in attributes must be wrapped in quotes for now."),
    }
}

/// Handle the `index_type` index key.
fn index_type(attr: &syn::Meta, attr_name: &str, index_root: &mut IndexRoot) {
    let index_type: String = extract_meta_kv(attr, attr_name);
    index_root.index = index_type;
    index_root.index_field_was_declared = true;
}

/// Set the index field & type for the given index "keys" document.
///
/// This routine will set the field name & field type in the given "keys" document in a way
/// that is compatible with MongoDB.
fn set_index_field_and_type(field_name: &str, index_type: &str, keys: &mut Document) {
    match index_type {
        "asc" => keys.insert(field_name, 1i32),
        "dsc" => keys.insert(field_name, -1i32),
        "2d" => keys.insert(field_name, "2d"),
        "2dsphere" => keys.insert(field_name, "2dsphere"),
        "text" => keys.insert(field_name, "text"),
        "geoHaystack" => keys.insert(field_name, "geoHaystack"),
        "hashed" => keys.insert(field_name, "hashed"),
        _ => panic!(msg::MODEL_ATTR_INDEX_TYPE_ALLOWED_VALUES),
    };
}

/// Handle the `with` index key.
fn index_with(index_with_container: &syn::Meta, index_root: &mut IndexRoot) {
    // Collect internals of `index(with(...))` attr as IndexWith model.
    let index_with_model = match index_with_container {
        syn::Meta::List(meta_list) => {
            // Ensure the `with(...)` declaration has the expected length of 2.
            if meta_list.nested.len() != 2 {
                panic!(msg::MODEL_ATTR_INDEX_WITH_FORM);
            }

            // Parse out contents of attr into IndexWith data model.
            meta_list
                .nested
                .iter()
                .fold(IndexWith::default(), |mut acc, nested| match nested {
                    syn::NestedMeta::Meta(meta) => match meta {
                        syn::Meta::NameValue(name_val) => match name_val.ident.to_string().as_str()
                        {
                            "field" => {
                                let val =
                                    extract_lit_str(&name_val.lit, msg::MODEL_ATTR_INDEX_WITH_FORM);
                                acc.field = val;
                                acc
                            }
                            "index" => {
                                let val =
                                    extract_lit_str(&name_val.lit, msg::MODEL_ATTR_INDEX_WITH_FORM);
                                acc.index = val;
                                acc
                            }
                            _ => panic!(msg::MODEL_ATTR_INDEX_WITH_FORM),
                        },
                        _ => panic!(msg::MODEL_ATTR_INDEX_WITH_FORM),
                    },
                    _ => panic!(msg::MODEL_ATTR_INDEX_WITH_FORM),
                })
        }
        _ => panic!(msg::MODEL_ATTR_INDEX_WITH_FORM),
    };

    // Use IndexWith model to generate new index entry.
    set_index_field_and_type(
        &index_with_model.field,
        &index_with_model.index,
        &mut index_root.with,
    );
}

/// Get the name that is to be used for the target field inside of the MongoDB database.
fn get_db_field_name(field: &syn::Field) -> String {
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
                let serde_name = list
                    .nested
                    .iter()
                    .by_ref()
                    .filter_map(|nested_meta| match nested_meta {
                        syn::NestedMeta::Meta(meta) => Some(meta),
                        _ => None,
                    })
                    .filter_map(|meta| match meta {
                        syn::Meta::NameValue(kv) => Some(kv),
                        _ => None,
                    })
                    .filter_map(|kv| {
                        if kv.ident != "rename" {
                            return None;
                        }
                        match kv.lit {
                            syn::Lit::Str(ref lit) => Some(lit.value()),
                            _ => None,
                        }
                    })
                    .last();
                match serde_name {
                    Some(name) => name,
                    _ => continue,
                }
            }
            _ => continue,
        };
        // NOTE WELL: hitting this return statement means that we found a serde `rename` for the field.
        return serde_name;
    }

    // Serde rename not found, so use field ident.
    // **NOTE:** a panic should never actually happen here, as we are
    // validating ahead of time that the target model's fields are named.
    field
        .ident
        .as_ref()
        .expect("Model fields must be named.")
        .to_string()
}

/// Extract a string from the given lit value, else panic with given message.
fn extract_lit_str(lit: &syn::Lit, msg: &'static str) -> String {
    match lit {
        syn::Lit::Str(litstr) => litstr.value(),
        _ => panic!(msg),
    }
}

/// An internal type used to model the declared root index field & type.
#[derive(Default)]
struct IndexRoot {
    pub field: String,
    pub embedded: Option<String>,
    pub index: String,
    pub index_field_was_declared: bool,
    pub with: Document,
}

impl IndexRoot {
    /// Consume self and return the finalized index keys for this index.
    pub fn final_keys(mut self) -> Document {
        // Insert initial root field for the index.
        let mut keys = Document::new();
        if let Some(embedded) = self.embedded {
            self.field.push('.');
            self.field.push_str(embedded.as_str());
        }
        set_index_field_and_type(&self.field, &self.index, &mut keys);

        // Added all other accumulated keys to the keys doc.
        for (key, val) in self.with {
            keys.insert(key, val);
        }

        keys
    }
}

/// An internal type used to model a `#[model(index(with(...)))]` attr's contents.
#[derive(Default)]
struct IndexWith {
    pub field: String,
    pub index: String,
}

/// An internal type used to model a `#[model(index(weight(...)))]` attr's contents.
#[derive(Default)]
struct IndexWeight {
    pub field: String,
    pub weight: i32,
}
