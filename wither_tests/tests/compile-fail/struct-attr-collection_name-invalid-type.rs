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

#[derive(Serialize, Deserialize, Model)]
#[model(collection_name=true)]
struct BadModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    id: Option<mongodb::oid::ObjectId>,
}
//~^^^^^^ ERROR: proc-macro derive panicked
//~| HELP: Only string literals are supported as named values in `Model` attributes.
