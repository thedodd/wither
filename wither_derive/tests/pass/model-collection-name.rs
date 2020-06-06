use serde::{Serialize, Deserialize};
use wither::prelude::*;

#[derive(Serialize, Deserialize, Model, Default)]
#[model(collection_name="derivations")]
struct DerivedModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

fn main() {
    let _model = DerivedModel::default();
    assert_eq!(DerivedModel::COLLECTION_NAME, "derivations");
}
