use inflector::Inflector;
use syn;

use ::msg;

/// All `Model` struct attributes which have been accumulated from the target struct.
#[derive(Default)]
pub(crate) struct MetaModelStructData {
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
                        _ => panic!(msg::MODEL_ATTR_FORM),
                    }).filter_map(|meta| match meta {
                        syn::Meta::NameValue(kv) => Some(kv),
                        _ => panic!(msg::MODEL_STRUCT_ATTR_FORM),
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
                _ => panic!(msg::MODEL_ATTR_FORM),
            };
            acc
        });

        // If collection name is default "", then use the struct's ident.
        if data.collection_name.len() == 0 {
            data.collection_name = target_ident.to_string().to_table_case().to_plural();
        }
        data
    }
}
