use mongodb::coll::options::IndexModel;
use syn::{self, DeriveInput};

use model_field::MetaModelFieldData;
use model_struct::MetaModelStructData;
use msg;

/// A meta representation of the `Model` derivation target.
pub(crate) struct MetaModel {
    ident: syn::Ident,
    generics: syn::Generics,
    struct_fields: syn::FieldsNamed,
    struct_data: MetaModelStructData,
    field_data: MetaModelFieldData,
}

impl MetaModel {
    /// Create a new instance.
    pub fn new(input: DeriveInput) -> Self {
        // The target's ident.
        let ident = input.ident;
        let generics = input.generics;

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

        MetaModel{ident, generics, struct_fields, struct_data, field_data}
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
            panic!(msg::ID_FIELD_SERDE_REQ);
        }
        panic!(msg::ID_FIELD_SERDE_REQ);
    }

    /// The derived indexes on this model.
    pub fn indexes(&self) -> Vec<IndexModel> {
        self.field_data.indexes.clone()
    }

    /// The target struct's ident.
    pub fn struct_name(&self) -> &syn::Ident {
        &self.ident
    }

    /// The target struct's generics.
    pub fn generics(&self) -> &syn::Generics {
        &self.generics
    }

    /// The write replication settings for this model. Defaults to `1`.
    pub fn write_concern_w(&self) -> i32 {
        self.struct_data.wc_replication
    }

    /// The write concern timeout settings for this model. Defaults to `0`.
    pub fn write_concern_w_timeout(&self) -> i32 {
        self.struct_data.wc_timeout
    }

    /// The write concern journal settings for this model. Defaults to `true`.
    pub fn write_concern_j(&self) -> bool {
        self.struct_data.wc_journaling
    }

    /// The write concern fsync settings for this model. Defaults to `false`.
    pub fn write_concern_fsync(&self) -> bool {
        self.struct_data.wc_fsync
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
