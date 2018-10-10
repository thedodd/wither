#[macro_use]
extern crate bson;
extern crate compiletest_rs as compiletest;
extern crate mongodb;
extern crate serde;
#[macro_use(Serialize, Deserialize)]
extern crate serde_derive;
extern crate wither;
#[macro_use(Model)]
extern crate wither_derive;

use wither::prelude::*;
use mongodb::coll::options::{IndexModel, IndexOptions};

#[derive(Serialize, Deserialize, Model)]
#[model(collection_name="valid_data_models", skip_serde_checks="false")]
struct ValidDataModel0 {
    /// The ID of the model.
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    id: Option<bson::oid::ObjectId>,

    /// A field to test base line index options & bool fields with `true`.
    #[model(index(
        direction="asc",
        background="true", sparse="true", unique="true",
        expire_after_seconds="15", name="field0", version="1", default_language="en_us",
        language_override="en_us", text_version="1", sphere_version="1", bits="1", max="10.0", min="1.0", bucket_size="1",
    ))]
    field0: String,

    /// A field to test bool fields with `false`.
    #[model(index(
        direction="dsc",
        background="false", sparse="false", unique="false",
    ))]
    field1: String,

    /// A field to test `weights` option.
    /// TODO:
    /// - ensure weights are compiling correctly.
    /// - fix issues with storage_engine. Apparently needs to be a doc.
    #[model(index(direction="dsc", /* weights="", storage_engine="wt" */))]
    field2: String,
}


#[test]
fn test_model_collection_name() {
    assert_eq!("valid_data_models", ValidDataModel0::COLLECTION_NAME);
}

#[test]
fn test_model_indexes() {
    let indexes = ValidDataModel0::indexes();
    assert_eq!(3, indexes.len());

    let (idx1, idx2, idx3) = (indexes[0].clone(), indexes[1].clone(), indexes[2].clone());
    assert_eq!(idx1.keys, doc!{"field0": 1i32});
    assert_eq!(idx1.options.background, Some(true));
    assert_eq!(idx1.options.bits, Some(1i32));
    assert_eq!(idx1.options.bucket_size, Some(1i32));
    assert_eq!(idx1.options.default_language, Some("en_us".to_string()));
    assert_eq!(idx1.options.expire_after_seconds, Some(15i32));
    assert_eq!(idx1.options.language_override, Some("en_us".to_string()));
    assert_eq!(idx1.options.max, Some(10.0));
    assert_eq!(idx1.options.min, Some(1.0));
    assert_eq!(idx1.options.name, Some("field0".to_string()));
    assert_eq!(idx1.options.sparse, Some(true));
    assert_eq!(idx1.options.sphere_version, Some(1i32));
    assert_eq!(idx1.options.storage_engine, None);
    assert_eq!(idx1.options.text_version, Some(1i32));
    assert_eq!(idx1.options.unique, Some(true));
    assert_eq!(idx1.options.version, Some(1i32));
    assert_eq!(idx1.options.weights, None);

    assert_eq!(idx2.keys, doc!{"field1": -1i32});
    assert_eq!(idx2.options.background, Some(false));
    assert_eq!(idx2.options.bits, None);
    assert_eq!(idx2.options.bucket_size, None);
    assert_eq!(idx2.options.default_language, None);
    assert_eq!(idx2.options.expire_after_seconds, None);
    assert_eq!(idx2.options.language_override, None);
    assert_eq!(idx2.options.max, None);
    assert_eq!(idx2.options.min, None);
    assert_eq!(idx2.options.name, None);
    assert_eq!(idx2.options.sparse, Some(false));
    assert_eq!(idx2.options.sphere_version, None);
    assert_eq!(idx2.options.storage_engine, None);
    assert_eq!(idx2.options.text_version, None);
    assert_eq!(idx2.options.unique, Some(false));
    assert_eq!(idx2.options.version, None);
    assert_eq!(idx2.options.weights, None);

    assert_eq!(idx3.keys, doc!{"field2": -1i32});
    assert_eq!(idx3.options.background, None);
    assert_eq!(idx3.options.bits, None);
    assert_eq!(idx3.options.bucket_size, None);
    assert_eq!(idx3.options.default_language, None);
    assert_eq!(idx3.options.expire_after_seconds, None);
    assert_eq!(idx3.options.language_override, None);
    assert_eq!(idx3.options.max, None);
    assert_eq!(idx3.options.min, None);
    assert_eq!(idx3.options.name, None);
    assert_eq!(idx3.options.sparse, None);
    assert_eq!(idx3.options.sphere_version, None);
    assert_eq!(idx3.options.storage_engine, None);
    assert_eq!(idx3.options.text_version, None);
    assert_eq!(idx3.options.unique, None);
    assert_eq!(idx3.options.version, None);
    assert_eq!(idx3.options.weights, None);
}
