use inflector::Inflector;
use syn;

use ::msg;

/// All `Model` struct attributes which have been accumulated from the target struct.
pub(crate) struct MetaModelStructData {
    /// The name to be used for the model's collection.
    pub collection_name: String,

    /// An attribute to control whether or not this derive macro will check for serde attributes.
    pub skip_serde_checks: bool,

    /// An attribute to control write concern replication.
    pub wc_replication: i32,

    /// An attribute to control write concern replication timeout.
    pub wc_timeout: i32,

    /// An attribute to control write concern journaling.
    pub wc_journaling: bool,

    /// An attribute to control write concern fsync.
    pub wc_fsync: bool,
}

impl Default for MetaModelStructData {
    fn default() -> Self {
        MetaModelStructData {
            collection_name: Default::default(),
            skip_serde_checks: Default::default(),
            wc_replication: 1i32,
            wc_timeout: Default::default(),
            wc_journaling: true,
            wc_fsync: Default::default(),
        }
    }
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
            let meta_name = meta.name().to_string();

            // If we are not looking at a `model` attr, then skip.
            match meta_name.as_str() {
                "model" => {
                    unpack_model_attr(&meta, &mut acc);
                    acc
                },
                _ => acc,
            }
        });

        // If collection name is default "", then use the struct's ident.
        if data.collection_name.len() == 0 {
            data.collection_name = target_ident.to_string().to_table_case().to_plural();
        }
        data
    }

}

/// Unpack the data from any struct level `model` attrs.
fn unpack_model_attr(meta: &syn::Meta, struct_data: &mut MetaModelStructData) {
    // Unpack the inner attr's components.
    match meta {
        // Model attr must be a list.
        syn::Meta::List(list) => list.nested.iter().by_ref()
            .filter_map(|nested_meta| match nested_meta {
                syn::NestedMeta::Meta(meta) => Some(meta),
                _ => panic!(msg::MODEL_ATTR_FORM),
            }).for_each(|innermeta| {
                match innermeta {
                    syn::Meta::Word(ident) => handle_ident_attr(ident, struct_data),
                    syn::Meta::NameValue(kv) => handle_kv_attr(kv, struct_data),
                    _ => panic!(format!("Unrecognized struct-level `Model` attribute '{}'.", innermeta.name())),
                }
            }),
        _ => panic!(msg::MODEL_ATTR_FORM),
    };
}

fn handle_kv_attr(kv: &syn::MetaNameValue, struct_data: &mut MetaModelStructData) {
    let ident = kv.ident.to_string();
    match &kv.lit {
        syn::Lit::Str(ref val) => {
            let value = val.value();
            match ident.as_str() {
                "collection_name" => {
                    struct_data.collection_name = value;
                    if struct_data.collection_name.len() < 1 {
                        panic!("The `Model` struct attribute 'collection_name' may not have a zero-length value.");
                    }
                },
                "wc_replication" => {
                    let parsed = value.parse::<i32>().expect("Value for `model(wc_replication)` must be an `i32` wrapped in a string.");
                    struct_data.wc_replication = parsed;
                },
                "wc_timeout" => {
                    let parsed = value.parse::<i32>().expect("Value for `model(wc_timeout)` must be an `i32` wrapped in a string.");
                    struct_data.wc_timeout = parsed;
                },
                "wc_journaling" => {
                    let parsed = value.parse::<bool>().expect("Value for `model(wc_journaling)` must be a `bool` wrapped in a string.");
                    struct_data.wc_journaling = parsed;
                },
                "wc_fsync" => {
                    let parsed = value.parse::<bool>().expect("Value for `model(wc_fsync)` must be a `bool` wrapped in a string.");
                    struct_data.wc_fsync = parsed;
                },
                _ => panic!(format!("Unrecognized struct-level `Model` attribute '{}'.", ident)),
            }
        },
        _ => panic!("Only string literals are supported as named values in `Model` attributes."),
    }
}

fn handle_ident_attr(ident: &syn::Ident, struct_data: &mut MetaModelStructData) {
    let ident = ident.to_string();
    match ident.as_str() {
        "skip_serde_checks" => {
            struct_data.skip_serde_checks = true;
        },
        _ => panic!(format!("Unrecognized struct-level `Model` attribute '{}'.", ident)),
    }
}
