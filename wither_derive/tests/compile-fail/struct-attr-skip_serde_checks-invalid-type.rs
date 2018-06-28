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
#[model(skip_serde_checks="abc")]
struct BadModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    id: Option<bson::oid::ObjectId>,
}
//~^^^^^^ ERROR: proc-macro derive panicked
//~| HELP: Value for `skip_serde_checks` must be parseable as a `bool`.
