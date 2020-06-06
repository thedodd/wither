#![cfg(feature="sync")]

use std::env;

use chrono::{self, TimeZone};
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use wither::bson::doc;
use wither::bson::oid::ObjectId;
use wither::mongodb::sync::{Client, Database};
use wither::prelude::*;

lazy_static!{
    static ref DB: Database = {
        let host = env::var("HOST").expect("Environment variable HOST must be defined.");
        let port = env::var("PORT").expect("Environment variable PORT must be defined.")
            .parse::<u32>().expect("Environment variable PORT must be an instance of `u32`.");
        let connection_string = format!("mongodb://{}:{:?}/", host, port);
        Client::with_uri_str(&connection_string)
            .expect("Expected MongoDB instance to be available for testing.")
            .database("witherTestDB")
    };
}

//////////////////////////////////////////////////////////////////////////////
// User //////////////////////////////////////////////////////////////////////

#[derive(ModelSync, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[model(collection_name="users")]
#[model(index(keys=r#"doc!{"email": 1}"#, options=r#"doc!{"name": "unique-email", "unique": true, "background": true}"#))]
pub struct User {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl MigratingSync for User {
    fn migrations() -> Vec<Box<dyn MigrationSync>> {
        vec![
            // This migration doesn't really do much. Just exercises the system.
            Box::new(wither::IntervalMigrationSync{
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

#[derive(ModelSync, Serialize, Deserialize, Debug, Clone)]
#[model(collection_name="users_bad_migrations")]
#[model(index(keys=r#"doc!{"email": 1}"#, options=r#"doc!{"name": "unique-email", "unique": true, "background": true}"#))]
pub struct UserModelBadMigrations {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl MigratingSync for UserModelBadMigrations {
    fn migrations() -> Vec<Box<dyn MigrationSync>> {
        vec![
            // This migration doesn't really do much. Just exercises the system.
            Box::new(wither::IntervalMigrationSync{
                name: String::from("test-migration"),
                threshold: chrono::Utc.ymd(2100, 1, 1).and_hms(1, 0, 0),
                filter: doc!{"email": doc!{"$exists": true}},
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
#[derive(Default)]
pub struct Fixture;

//////////////////////////////////////////////////////////////////////////////
// Public Builder Interface //////////////////////////////////////////////////

impl Fixture {
    /// Create a new fixture.
    pub fn new() -> Self {
        Fixture::default()
    }

    // /// Remove all documents & indexes from the collections of the data models used by this harness.
    // pub fn with_empty_collections(self) -> Self {
    //     DB.clone().collection(User::COLLECTION_NAME).drop(None).expect("failed to drop collection");
    //     DB.clone().collection(UserModelBadMigrations::COLLECTION_NAME).drop(None).expect("failed to drop collection");
    //     self
    // }

    /// Drop the database which is used by this harness.
    pub fn with_dropped_database(self) -> Self {
        DB.clone().drop(None).expect("failed to drop database");
        self
    }

    /// Sync all of the data models used by this harness.
    pub fn with_synced_models(self) -> Self {
        User::sync(DB.clone()).expect("failed to sync `User` model");
        self
    }
}

//////////////////////////////////////////////////////////////////////////////
// Public Dependencies Interface /////////////////////////////////////////////

impl Fixture {
    /// Get a handle to the database used by this harness.
    pub fn get_db(&self) -> Database {
        DB.clone()
    }
}
