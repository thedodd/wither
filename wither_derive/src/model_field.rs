use std::fmt::Display;
use std::str::FromStr;

use bson::Document;
use mongodb::coll::options::{IndexModel, IndexOptions};
use syn;

use msg;

/// The `Model` field attributes which have been accumulated from the target model.
#[derive(Default)]
pub(crate) struct MetaModelFieldData {
    pub indexes: Vec<IndexModel>,
}

impl MetaModelFieldData {
    /// Extract needed data from the target model's field attributes. Call `.finish()` to finalize.
    pub fn build(struct_fields: &syn::FieldsNamed) -> MetaModelFieldDataBuilder {
        MetaModelFieldDataBuilder(struct_fields)
    }
}

/// A builder for the `MetaModelFieldData` struct.
pub(crate) struct MetaModelFieldDataBuilder<'a>(&'a syn::FieldsNamed);

impl<'a> MetaModelFieldDataBuilder<'a> {
    /// Finish the building process.
    pub fn finish(self) -> MetaModelFieldData {
        // Iterate over fields of the target model & accumulate needed data.
        self.0.named.iter()
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
                        _ => panic!(msg::MODEL_ATTR_FORM),
                    };
                    let inner_meta = meta_list.nested.iter().map(|nested| {
                        match nested {
                            syn::NestedMeta::Meta(meta) => meta,
                            _ => panic!(msg::MODEL_ATTR_FORM),
                        }
                    }).collect::<Vec<&syn::Meta>>();

                    // Iterate over the inner meta elements & handle as needed.
                    for inner in inner_meta {
                        let attr_name = inner.name().to_string();
                        match attr_name.as_ref() {
                            // Handle index attrs.
                            "index" => {
                                let model = self.build_index_model(field, inner);
                                acc.indexes.push(model);
                            },
                            _ => panic!("Unrecognized `model` field attribute '{}'.", attr_name),
                        }
                    }
                }
                acc
            })
    }

    /// Build an `IndexModel` from the metadata in the given attr meta object.
    fn build_index_model(&self, field: &syn::Field, index_container: &syn::Meta) -> IndexModel {
        // Collect the internals of the `index` attr as a vector of additional meta elements.
        let index_attrs = match index_container {
            syn::Meta::List(meta_list) => meta_list.nested.iter().map(|nested| {
                match nested {
                    syn::NestedMeta::Meta(meta) => meta,
                    _ => panic!(msg::MODEL_ATTR_INDEX_ELEMENT_FORM),
                }
            }).collect::<Vec<&syn::Meta>>(),
            _ => panic!(msg::MODEL_ATTR_INDEX_FORM),
        };

        // A few symbols needed for building a new index model.
        let mut keys = doc!{};
        let mut opts = IndexOptions::new();
        let name = self.get_db_field_name(field);

        // Iterator over index metadata.
        for meta_elem in index_attrs {
            let meta_elem_name = meta_elem.name().to_string();
            match meta_elem_name.as_ref() {
                "direction" => self.index_direction(meta_elem, &mut keys, &meta_elem_name, &name),
                // TODO: come back and implement this.
                // "with" => self.index_with(meta_elem, &mut keys),
                "background" => opts.background = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                "expire_after_seconds" => opts.expire_after_seconds = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                "name" => opts.name = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                "sparse" => opts.sparse = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                "storage_engine" => opts.storage_engine = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                "unique" => opts.unique = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                "version" => opts.version = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                "default_language" => opts.default_language = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                "language_override" => opts.language_override = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                "text_version" => opts.text_version = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                // TODO: come back and implement this.
                // "weights" => self.index_weights(meta_elem, &mut opts, &meta_elem_name),
                "sphere_version" => opts.sphere_version = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                "bits" => opts.bits = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                "max" => opts.max = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                "min" => opts.min = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                "bucket_size" => opts.bucket_size = Some(self.extract_meta_kv(meta_elem, &meta_elem_name)),
                _ => panic!("Unrecognized `#[model(index(...))]` attribute '{}'.", meta_elem_name),
            };
        }
        IndexModel::new(keys, Some(opts))
    }

    /// A generic value extraction method for extracting the value of a `syn::Meta::NameValue` field.
    fn extract_meta_kv<T>(&self, attr: &syn::Meta, attr_name: &str) -> T
        where T: FromStr,
              <T as FromStr>::Err: Display,
    {
        let name_value = match attr {
            syn::Meta::NameValue(kv) => kv,
            _ => panic!(format!("The index `#[model(index({}))]` key must be a name-value pair.", attr_name)),
        };
        match name_value.lit {
            syn::Lit::Str(ref val) => match val.value().parse() {
                Ok(val) => val,
                Err(err) => panic!(format!("Invalid type provided for as value for the `#[model(index({}))]` attribute. Must be needed type wrapped in quotes. {}", attr_name, err)),
            },
            _ => panic!("Named values in attributes must be wrapped in quotes for now."),
        }
    }

    /// Handle the `direction` index key.
    fn index_direction(&self, attr: &syn::Meta, keys: &mut Document, attr_name: &str, field_name: &str) {
        let lit_val: String = self.extract_meta_kv(attr, attr_name);
        if lit_val != "asc" && lit_val != "dsc" {
            panic!(msg::MODEL_ATTR_INDEX_DIRECTION_ALLOWED_VALUES);
        }
        keys.insert(field_name, lit_val);
    }

    // TODO: come back and implement this.
    // /// Handle the `with` index key.
    // fn index_with(&self, attr: &syn::Meta, keys: &mut Document, attr_name: &str) {
    //     // Option<()>
    // }

    // TODO: come back and implement this.
    // /// Handle the `weights` index option.
    // fn index_weights(&self, attr: &syn::Meta, opts: &mut IndexOptions, attr_name: &str) {
    //     let lit_val: Document = self.extract_meta_kv(attr, attr_name);
    //     opts.weights = Some(lit_val);
    // }

    /// Get the name that is to be used for the target field inside of the MongoDB database.
    fn get_db_field_name(&self, field: &syn::Field) -> String {
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
            // NOTE WELL: hitting this return statement means that we found a serde `rename` for the field.
            return serde_name;
        }

        // Serde rename not found, so use field ident.
        // **NOTE:** a panic should never actually happen here, as we are
        // validating ahead of time that the target model's fields are named.
        field.ident.as_ref().expect("Model fields must be named.").to_string()
    }
}
