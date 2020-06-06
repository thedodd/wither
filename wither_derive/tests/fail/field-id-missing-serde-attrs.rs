use serde::{Serialize, Deserialize};
use wither::Model;

#[derive(Serialize, Deserialize, Model)]
struct BadModel {
    id: Option<wither::bson::oid::ObjectId>,
}

fn main() {}
