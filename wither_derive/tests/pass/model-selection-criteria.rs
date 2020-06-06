use serde::{Serialize, Deserialize};
use wither::prelude::*;

#[derive(Serialize, Deserialize, Model, Default)]
#[model(selection_criteria="DerivedModel::get_selection_criteria")]
struct DerivedModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

fn main() {
    let _model = DerivedModel::default();
    assert_eq!(DerivedModel::selection_criteria(), Some(wither::mongodb::options::SelectionCriteria::ReadPreference(
        wither::mongodb::options::ReadPreference::Primary
    )));
}

impl DerivedModel {
    pub fn get_selection_criteria() -> wither::mongodb::options::SelectionCriteria {
        wither::mongodb::options::SelectionCriteria::ReadPreference(
            wither::mongodb::options::ReadPreference::Primary
        )
    }
}
