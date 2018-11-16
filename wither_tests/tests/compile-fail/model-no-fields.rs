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
struct BadModel;
//~^^ ERROR: proc-macro derive panicked
//~| HELP: Structs used as a `Model` must have named fields.
