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
    #[serde(rename="_bad", skip_serializing_if="Option::is_none")]
    id: Option<bson::oid::ObjectId>,
}
//~^^^^^ ERROR: proc-macro derive panicked
//~| HELP: A `Model`'s 'id' field must have a serde `rename` attribute with a value of `"_id"`.
