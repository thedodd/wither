use chrono::{self, TimeZone};
use serde::{Serialize, Deserialize};
use wither::prelude::*;
use wither::bson::doc;
use wither::bson::oid::ObjectId;

pub mod fixture;

pub use self::fixture::Fixture;

//////////////////////////////////////////////////////////////////////////////
// User //////////////////////////////////////////////////////////////////////

#[derive(Model, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[model(collection_name="users")]
#[model(index(keys=r#"doc!{"email": 1}"#, options=r#"doc!{"name": "unique-email", "unique": true, "background": true}"#))]
pub struct User {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl Migrating for User {
    fn migrations() -> Vec<Box<dyn wither::Migration>> {
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

#[derive(Model, Serialize, Deserialize, Debug, Clone)]
#[model(collection_name="users_bad_migrations")]
#[model(index(keys=r#"doc!{"email": 1}"#, options=r#"doc!{"name": "unique-email", "unique": true, "background": true}"#))]
pub struct UserModelBadMigrations {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl Migrating for UserModelBadMigrations {
    fn migrations() -> Vec<Box<dyn wither::Migration>> {
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
