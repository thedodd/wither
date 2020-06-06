use serde::{Serialize, Deserialize};
use wither::Model;

#[derive(Serialize, Deserialize, Model)]
#[model(collection_name="")]
struct BadModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

fn main() {}
