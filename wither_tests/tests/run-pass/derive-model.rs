extern crate compiletest_rs as compiletest;
#[macro_use]
extern crate mongodb;
extern crate serde;
#[macro_use(Serialize, Deserialize)]
extern crate serde_derive;
extern crate wither;
#[macro_use(Model)]
extern crate wither_derive;

use wither::Model;
use mongodb::coll::options::IndexModel;

#[derive(Serialize, Deserialize, Model)]
#[model(collection_name="valid_data_models_0")]
struct ValidDataModel0 {
    /// The ID of the model.
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    id: Option<mongodb::oid::ObjectId>,

    /// A field to test base line index options & bool fields with `true`.
    #[model(index(
        index_type="asc",
        background="true", sparse="true", unique="true",
        expire_after_seconds="15", name="field0", storage_engine="wt", version="1", default_language="en_us",
        language_override="en_us", text_version="1", sphere_version="1", bits="1", max="10.0", min="1.0", bucket_size="1",
    ))]
    field0: String,

    /// A field to test bool fields with `false`.
    #[model(index(
        index_type="dsc",
        background="false", sparse="false", unique="false",
    ))]
    field1: String,

    /// A field to test `weights` option.
    #[model(index(index_type="dsc", /*weights=""*/))] // TODO: ensure weights are compiling correctly.
    field2: String,
}

fn main() {}
