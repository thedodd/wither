extern crate bson;
extern crate compiletest_rs as compiletest;
extern crate mongodb;
extern crate serde;
#[macro_use(Serialize, Deserialize)]
extern crate serde_derive;
extern crate wither;
#[macro_use(Model)]
extern crate wither_derive;

use wither::Model;

#[derive(Serialize, Deserialize, Model)]
#[model(collection_name="valid_data_models_0", skip_serde_checks="false")]
struct ValidDataModel0 {
    /// The ID of the model.
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    id: Option<bson::oid::ObjectId>,

    /// Another field being indexed.
    #[model(index(direction="asc"))]
    field0: String,
}

fn main() {}
