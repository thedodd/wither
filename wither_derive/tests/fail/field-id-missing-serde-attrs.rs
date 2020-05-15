use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use wither::Model;

#[derive(Serialize, Deserialize, Model)]
struct BadModel {
    id: Option<ObjectId>,
}

fn main() {}
