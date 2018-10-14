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
use mongodb::coll::options::IndexModel;

#[derive(Model)]
#[model(skip_serde_checks="true")]
struct BadModel {
    id: Option<bson::oid::ObjectId>,
}
//~^^^^^ 15:10: 15:15: the trait bound `BadModel: serde::Deserialize<'a>` is not satisfied [E0277]
//~^^^^^^ 15:10: 15:15: the trait bound `BadModel: serde::Serialize` is not satisfied [E0277]
