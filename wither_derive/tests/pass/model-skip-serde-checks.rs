//! The `id` field does not have the required serde attrs, but the check is disabled.
use serde::{Serialize, Deserialize};
use wither::Model;

#[derive(Serialize, Deserialize, Model)]
#[model(skip_serde_checks)]
struct DerivedModel {
    pub id: Option<wither::bson::oid::ObjectId>,
}

fn main() {
}
