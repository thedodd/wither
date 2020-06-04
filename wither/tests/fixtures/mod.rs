use chrono::{self, TimeZone};
use serde::{Serialize, Deserialize};
use wither::prelude::*;
use wither::bson::doc;
use wither::bson::oid::ObjectId;
use wither::mongodb::options::IndexModel;

pub mod fixture;

pub use self::fixture::Fixture;

//////////////////////////////////////////////////////////////////////////////
// User //////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct User {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl Model for User {
    const COLLECTION_NAME: &'static str = "users";

    fn id(&self) -> Option<ObjectId> {
        return self.id.clone();
    }

    fn set_id(&mut self, oid: ObjectId) {
        self.id = Some(oid);
    }

    fn indexes() -> Vec<IndexModel> {
        vec![IndexModel{
            keys: doc!{"email": 1},
            options: Some(doc!{"name": "unique-email", "unique": true, "background": true}),
        }]
    }
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserModelBadMigrations {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl Model for UserModelBadMigrations {
    const COLLECTION_NAME: &'static str = "users_bad_migrations";

    fn id(&self) -> Option<ObjectId> {
        return self.id.clone();
    }

    fn set_id(&mut self, oid: ObjectId) {
        self.id = Some(oid);
    }

    fn indexes() -> Vec<IndexModel> {
        vec![IndexModel{
            keys: doc!{"email": 1},
            options: Some(doc!{"name": "unique-email", "unique": true, "background": true}),
        }]
    }
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
