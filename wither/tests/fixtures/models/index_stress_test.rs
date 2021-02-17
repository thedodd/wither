use serde::{Deserialize, Serialize};
use wither::bson::doc;
use wither::bson::oid::ObjectId;
use wither::prelude::*;

#[derive(Model, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[model(collection_name="indexStressTest")]
#[model(index(keys=r#"doc!{"i_0": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": -1}"#))]
#[model(index(keys=r#"doc!{"i_2": -1}"#))]
#[model(index(keys=r#"doc!{"i_3": -1}"#))]
#[model(index(keys=r#"doc!{"i_4": -1}"#))]
#[model(index(keys=r#"doc!{"i_5": -1}"#))]
#[model(index(keys=r#"doc!{"i_6": -1}"#))]
#[model(index(keys=r#"doc!{"i_7": -1}"#))]
#[model(index(keys=r#"doc!{"i_8": -1}"#))]
#[model(index(keys=r#"doc!{"i_9": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1}"#))]
#[model(index(keys=r#"doc!{"i_1": 1}"#))]
#[model(index(keys=r#"doc!{"i_2": 1}"#))]
#[model(index(keys=r#"doc!{"i_3": 1}"#))]
#[model(index(keys=r#"doc!{"i_4": 1}"#))]
#[model(index(keys=r#"doc!{"i_5": 1}"#))]
#[model(index(keys=r#"doc!{"i_6": 1}"#))]
#[model(index(keys=r#"doc!{"i_7": 1}"#))]
#[model(index(keys=r#"doc!{"i_8": 1}"#))]
#[model(index(keys=r#"doc!{"i_9": 1}"#))]
#[model(index(keys=r#"doc!{"i_0": -1, "i_1": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": -1, "i_2": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": -1, "i_3": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": -1, "i_4": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": -1, "i_5": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": -1, "i_6": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": -1, "i_7": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": -1, "i_8": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": -1, "i_9": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_1": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_2": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_3": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_4": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_5": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_6": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_7": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_8": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_9": -1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_1": 1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_2": 1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_3": 1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_4": 1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_5": 1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_6": 1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_7": 1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_8": 1}"#))]
#[model(index(keys=r#"doc!{"i_0": 1, "i_9": 1}"#))]
#[model(index(keys=r#"doc!{"i_1": -1, "i_0": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": -1, "i_2": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": -1, "i_3": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": -1, "i_4": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": -1, "i_5": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": -1, "i_6": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": -1, "i_7": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": -1, "i_8": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": -1, "i_9": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": 1, "i_0": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": 1, "i_2": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": 1, "i_3": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": 1, "i_4": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": 1, "i_5": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": 1, "i_6": -1}"#))]
#[model(index(keys=r#"doc!{"i_1": 1, "i_7": -1}"#))]
/// This model is just to stress-test the collection, it has the maximum number of indexes (64 so 63 custom indexes)
pub struct IndexStressTest {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub i_0: String,
    pub i_1: String,
    pub i_2: String,
    pub i_3: String,
    pub i_4: String,
    pub i_5: String,
    pub i_6: String,
    pub i_7: String,
    pub i_8: String,
    pub i_9: String,
}