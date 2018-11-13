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

use wither::Model;

#[derive(Serialize, Deserialize, Model)]
#[model(collection_name="")]
struct BadModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    id: Option<bson::oid::ObjectId>,
}
//~^^^^^^ ERROR: proc-macro derive panicked
//~| HELP: The `Model` struct attribute 'collection_name' may not have a zero-length value.
