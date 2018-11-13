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
struct BadModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_some")]
    id: Option<bson::oid::ObjectId>,
}
//~^^^^^ ERROR: proc-macro derive panicked
//~| HELP: A `Model`'s 'id' field must have a serde `skip_serializing_if` attribute with a value of `"Option::is_none"`.
