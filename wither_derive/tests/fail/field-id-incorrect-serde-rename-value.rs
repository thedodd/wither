use serde::{Serialize, Deserialize};
use wither::Model;

#[derive(Serialize, Deserialize, Model)]
struct BadModel {
    #[serde(rename="_bad", skip_serializing_if="Option::is_none")]
    id: Option<wither::bson::oid::ObjectId>,
}

fn main() {}
