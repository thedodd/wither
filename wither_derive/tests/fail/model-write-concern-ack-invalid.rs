use serde::{Serialize, Deserialize};
use wither::Model;

#[derive(Serialize, Deserialize, Model)]
#[model(write_concern(w=WriteConcern::Majority, w_timeout=10, journal=false))]
struct DerivedModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

fn main() {
    assert!(DerivedModel::write_concern().is_none());
}
