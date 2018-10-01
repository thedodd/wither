use quote::ToTokens;
use proc_macro2::TokenStream;

use mongodb::coll::options::{IndexModel, IndexOptions};
use serde_json;

pub struct Indexes(pub Vec<IndexModel>);

impl ToTokens for Indexes {
    /// Implement `ToTokens` for the `Indexes` type.
    ///
    /// This type is a simple wrapper around a `Vec<IndexModel>` and when it is converted to a
    /// token stream, it will simply be returned as the underlying `Vec` type.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // If we have no indexes, then code an empty vec.
        if self.0.len() == 0 {
            tokens.extend(quote!{vec![]});
            return;
        }

        // Else, build up a vector of token streams which we will interpolate later.
        let index_tokens = self.0.iter().map(|index| {
            // Desctructure variables needed for interpolation. Use struct destructuring syntax
            // to ensure we are not missing any fields.
            let doc_json = serde_json::to_string(&index.keys).expect("Expected valid BSON to also be valid JSON. If you are seeing this unexpectedly, then you should open an issue here: https://github.com/thedodd/wither/issues/new");
            let IndexOptions{
                background, expire_after_seconds, name, sparse, storage_engine, unique, version, default_language,
                language_override, text_version, weights, sphere_version, bits, max, min, bucket_size,
            } = index.options.clone();
            let background = option_to_tokens(background);
            let expire_after_seconds = option_to_tokens(expire_after_seconds);
            let name = option_to_tokens_for_string(name);
            let sparse = option_to_tokens(sparse);
            let storage_engine = option_to_tokens_for_string(storage_engine);
            let unique = option_to_tokens(unique);
            let version = option_to_tokens(version);
            let default_language = option_to_tokens_for_string(default_language);
            let language_override = option_to_tokens_for_string(language_override);
            let text_version = option_to_tokens(text_version);
            // let weights = None; // option_to_tokens(weights);
            let sphere_version = option_to_tokens(sphere_version);
            let bits = option_to_tokens(bits);
            let max = option_to_tokens(max);
            let min = option_to_tokens(min);
            let bucket_size = option_to_tokens(bucket_size);

            quote!(IndexModel{
                keys: serde_json::from_str(#doc_json).expect("Expected valid JSON to also be valid BSON. If you are seeing this unexpectedly, then you should open an issue here: https://github.com/thedodd/wither/issues/new"),
                options: IndexOptions{
                    background: #background, expire_after_seconds: #expire_after_seconds, name: #name, sparse: #sparse,
                    storage_engine: #storage_engine, unique: #unique, version: #version, default_language: #default_language,
                    language_override: #language_override, text_version: #text_version, weights: None, sphere_version: #sphere_version,
                    bits: #bits, max: #max, min: #min, bucket_size: #bucket_size,
                },
            })
        }).collect::<Vec<TokenStream>>();

        tokens.extend(quote!{
            use mongodb::coll::options::{IndexModel, IndexOptions};
            use serde_json;
            vec![
                #(#index_tokens),*
            ]
        });
    }
}

fn option_to_tokens<T: ToTokens>(target: Option<T>) -> TokenStream {
    match target {
        Some(t) => quote!(Some(#t)),
        None => quote!(None),
    }
}

fn option_to_tokens_for_string(target: Option<String>) -> TokenStream {
    match target {
        Some(t) => quote!(Some(String::from(#t))),
        None => quote!(None),
    }
}
