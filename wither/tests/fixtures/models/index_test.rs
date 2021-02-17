use serde::{Deserialize, Serialize};
use wither::bson::doc;
use wither::bson::oid::ObjectId;
use wither::prelude::*;

/// Index V1 has a basic index
#[derive(Model, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[model(collection_name = "indexTest")]
#[model(index(keys = r#"doc!{"i": 1}"#))]
pub struct IndexTestV1 {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub i: String,
}

/// Index V2 has the basic index with a different order
#[derive(Model, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[model(collection_name = "indexTest")]
#[model(index(keys = r#"doc!{"i": -1}"#))]
pub struct IndexTestV2 {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub i: String,
}
/// Index V3 has the basic index with an option
#[derive(Model, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[model(collection_name = "indexTest")]
#[model(index(keys = r#"doc!{"i": -1}"#, options = r#"doc!{"unique": true}"#))]
pub struct IndexTestV3 {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub i: String,
}

/// Index V4 has a different option
#[derive(Model, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[model(collection_name = "indexTest")]
#[model(index(keys = r#"doc!{"i": -1}"#, options = r#"doc!{"unique": true, "background": true}"#))]
pub struct IndexTestV4 {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub i: String,
}

/// Index V5 is the same as Index V4
#[derive(Model, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[model(collection_name = "indexTest")]
#[model(index(keys = r#"doc!{"i": -1}"#, options = r#"doc!{"unique": true, "background": true}"#))]
pub struct IndexTestV5 {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub i: String,
}

/// Index V6 has no indexes
#[derive(Model, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[model(collection_name = "indexTest")]
pub struct IndexTestV6 {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub i: String,
}
