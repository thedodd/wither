use std::env;

use lazy_static::lazy_static;
use wither::prelude::*;
use wither::mongodb::{Client, Database};

use super::{User, UserModelBadMigrations};

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

    /// Remove all documents & indexes from the collections of the data models used by this harness.
    pub fn with_empty_collections(self) -> Self {
        DB.clone().collection(User::COLLECTION_NAME).drop(None).expect("failed to drop collection");
        DB.clone().collection(UserModelBadMigrations::COLLECTION_NAME).drop(None).expect("failed to drop collection");
        self
    }

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
