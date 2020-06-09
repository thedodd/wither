use serde::{Serialize, Deserialize};
use wither::Model;

#[derive(Serialize, Deserialize, Model)]
struct BadModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    id: Option<String>,
}

fn main() {}
