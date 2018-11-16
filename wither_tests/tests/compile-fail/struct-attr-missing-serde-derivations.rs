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

#[derive(Model)]
#[model(skip_serde_checks)]
struct BadModel {
    id: Option<mongodb::oid::ObjectId>,
}
//~^^^^^ the trait bound `BadModel: serde::Deserialize<'a>` is not satisfied [E0277]
//~^^^^^^ the trait bound `BadModel: serde::Serialize` is not satisfied [E0277]
