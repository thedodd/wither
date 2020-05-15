use serde::{Serialize, Deserialize};
use wither::Model;

#[derive(Serialize, Deserialize, Model)]
#[model(selection_criteria="BadModel::get_selection_criteria")]
struct BadModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
}

fn main() {}
