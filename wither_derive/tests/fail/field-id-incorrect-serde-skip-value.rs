use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use wither::Model;

#[derive(Serialize, Deserialize, Model)]
struct BadModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_some")]
    id: Option<ObjectId>,
}

fn main() {}
