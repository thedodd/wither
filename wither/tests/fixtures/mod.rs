#![cfg(not(feature = "sync"))]

pub mod models;

use std::env;

use chrono::{self, TimeZone};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use wither::bson::doc;
use wither::bson::oid::ObjectId;
use wither::mongodb::{Client, Database};
use wither::prelude::*;

lazy_static! {
    static ref HOST: String = env::var("HOST").expect("environment variable HOST must be defined");
    static ref PORT: String = env::var("PORT").expect("environment variable PORT must be defined");
    static ref CONNECTION_STRING: String = format!("mongodb://{}:{}/", HOST.as_str(), PORT.as_str());
}

//////////////////////////////////////////////////////////////////////////////
// User //////////////////////////////////////////////////////////////////////

#[derive(Model, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[model(collection_name = "users")]
#[model(index(
    keys = r#"doc!{"email": 1}"#,
    options = r#"doc!{"name": "unique-email", "unique": true, "background": true}"#
))]
pub struct User {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl Migrating for User {
    fn migrations() -> Vec<Box<dyn wither::Migration<Self>>> {
        vec![
            // This migration doesn't really do much. Just exercises the system.
            Box::new(wither::IntervalMigration {
                name: String::from("test-migration"),
                threshold: chrono::Utc.ymd(2100, 1, 1).and_hms(1, 0, 0),
                filter: doc! {"email": doc!{"$exists": true}},
                set: Some(doc! {"testfield": "test"}),
                unset: None,
            }),
        ]
    }
}

//////////////////////////////////////////////////////////////////////////////
// UserModelBadMigrations ////////////////////////////////////////////////////

#[derive(Model, Serialize, Deserialize, Debug, Clone)]
#[model(collection_name = "users_bad_migrations")]
#[model(index(
    keys = r#"doc!{"email": 1}"#,
    options = r#"doc!{"name": "unique-email", "unique": true, "background": true}"#
))]
pub struct UserModelBadMigrations {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl Migrating for UserModelBadMigrations {
    fn migrations() -> Vec<Box<dyn wither::Migration<Self>>> {
        vec![
            // This migration doesn't really do much. Just exercises the system.
            Box::new(wither::IntervalMigration {
                name: String::from("test-migration"),
                threshold: chrono::Utc.ymd(2100, 1, 1).and_hms(1, 0, 0),
                filter: doc! {"email": doc!{"$exists": true}},
                set: None,
                unset: None,
            }),
        ]
    }
}

/// A singular type representing the various fixtures available in this harness.
///
/// This type represents some combination of desired states which this system's dependencies must
/// be in. Generally speaking, this represents the backend database; however it is not necessarily
/// limited to only the backend database.
pub struct Fixture {
    client: Client,
}

//////////////////////////////////////////////////////////////////////////////
// Public Builder Interface //////////////////////////////////////////////////

impl Fixture {
    /// Create a new fixture.
    pub async fn new() -> Self {
        let client = Client::with_uri_str(CONNECTION_STRING.as_str())
            .await
            .expect("failed to connect to database");
        Fixture { client }
    }

    // /// Remove all documents & indexes from the collections of the data models used by this harness.
    // pub fn with_empty_collections(self) -> Self {
    //     DB.clone().collection(User::COLLECTION_NAME).drop(None).expect("failed to drop collection");
    //     DB.clone().collection(UserModelBadMigrations::COLLECTION_NAME).drop(None).expect("failed to
    // drop collection");     self
    // }

    /// Drop the database which is used by this harness.
    pub async fn with_dropped_database(self) -> Self {
        self.get_db().drop(None).await.expect("failed to drop database");
        self
    }

    /// Sync all of the data models used by this harness.
    pub async fn with_synced_models(self) -> Self {
        User::sync(&self.get_db()).await.expect("failed to sync `User` model");
        self
    }
}

//////////////////////////////////////////////////////////////////////////////
// Public Dependencies Interface /////////////////////////////////////////////

impl Fixture {
    /// Get a handle to the database used by this harness.
    pub fn get_db(&self) -> Database {
        self.client.database("witherTestDB")
    }
}
