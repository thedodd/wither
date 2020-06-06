use serde::{Serialize, Deserialize};
use wither::bson::doc;
use wither::Model;

#[derive(Default, Serialize, Deserialize, Model)]
#[model(
    index(keys=r#"doc!{"id": 1}"#, options="doc!{}"),
    index(keys=r#"doc!{"id": -1}"#, options=r#"doc!{"unique": true}"#),
    index(keys=r#"doc!{"id.nested.field": 1}"#),
)]
struct DerivedModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

fn main() {
    let _model = DerivedModel::default();
    let indexes = DerivedModel::indexes();
    assert_eq!(indexes[0].keys, doc!{"id": 1});
    assert_eq!(indexes[0].options, Some(doc!{}));

    assert_eq!(indexes[1].keys, doc!{"id": -1});
    assert_eq!(indexes[1].options, Some(doc!{"unique": true}));

    assert_eq!(indexes[2].keys, doc!{"id.nested.field": 1});
    assert_eq!(indexes[2].options, None);
}
