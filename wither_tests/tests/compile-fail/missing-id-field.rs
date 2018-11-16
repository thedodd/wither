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
struct BadModel {
    name: String,
}
//~^^^^ ERROR: proc-macro derive panicked
//~| HELP: A `Model` struct must have a field named `id`, and it must have the following attribute: `#[serde(rename="_id", skip_serializing_if="Option::is_none")]`.
