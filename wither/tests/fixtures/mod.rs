use bson;
use chrono::{self, TimeZone};
use mongodb::coll::options::IndexModel;
use wither::{self, prelude::*};

pub mod fixture;

pub use self::fixture::Fixture;

//////////////////////////////////////////////////////////////////////////////
// User //////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl<'a> Model<'a> for User {

    const COLLECTION_NAME: &'static str = "users";

    fn id(&self) -> Option<bson::oid::ObjectId> {
        return self.id.clone();
    }

    fn set_id(&mut self, oid: bson::oid::ObjectId) {
        self.id = Some(oid);
    }

    fn indexes() -> Vec<IndexModel> {
        return vec![
            IndexModel{
                keys: doc!{"email" => 1},
                options: wither::basic_index_options("unique-email", true, Some(true), None, None),
            },
        ];
    }

    fn migrations() -> Vec<Box<wither::Migration>> {
        vec![
            // This migration doesn't really do much. Just exercises the system.
            Box::new(wither::IntervalMigration{
                name: String::from("test-migration"),
                threshold: chrono::Utc.ymd(2100, 1, 1).and_hms(1, 0, 0),
                filter: doc!{"email": doc!{"$exists": true}},
                set: Some(doc!{"testfield": "test"}),
                unset: None,
            }),
        ]
    }
}

//////////////////////////////////////////////////////////////////////////////
// UserModelBadMigrations ////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserModelBadMigrations {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl<'a> Model<'a> for UserModelBadMigrations {

    const COLLECTION_NAME: &'static str = "users_bad_migrations";

    fn id(&self) -> Option<bson::oid::ObjectId> {
        return self.id.clone();
    }

    fn set_id(&mut self, oid: bson::oid::ObjectId) {
        self.id = Some(oid);
    }

    fn indexes() -> Vec<IndexModel> {
        return vec![
            IndexModel{
                keys: doc!{"email" => 1},
                options: wither::basic_index_options("unique-email", true, Some(true), None, None),
            },
        ];
    }

    fn migrations() -> Vec<Box<wither::Migration>> {
        vec![
            // This migration doesn't really do much. Just exercises the system.
            Box::new(wither::IntervalMigration{
                name: String::from("test-migration"),
                threshold: chrono::Utc.ymd(2100, 1, 1).and_hms(1, 0, 0),
                filter: doc!{"email": doc!{"$exists": true}},
                set: None,
                unset: None,
            }),
        ]
    }
}

//////////////////////////////////////////////////////////////////////////////
// Derived Model /////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Model)]
#[model(collection_name="derivations")]
pub struct DerivedModel {
    /// The ID of the model.
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,

    /// A field to test base line index options & bool fields with `true`.
    #[model(index(
        index_type="asc",
        background="true", sparse="true", unique="true",
        expire_after_seconds="15", name="field0", version="1", default_language="en_us",
        language_override="en_us", text_version="1", sphere_version="1", bits="1", max="10.0", min="1.0", bucket_size="1",
    ))]
    pub field0: String,

    /// A field to test bool fields with `false`.
    #[model(index(
        index_type="dsc",
        background="false", sparse="false", unique="false",
    ))]
    pub field1: String,

    /// A field to test `weights` option.
    #[model(index(index_type="dsc", /* weights="", storage_engine="wt" */))]
    pub field2: String,
}
