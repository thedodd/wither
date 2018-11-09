use std::env;

use mongodb::{
    Client,
    db::{Database, ThreadedDatabase},
    ThreadedClient,
};
use wither::prelude::*;

use super::{
    User,
    UserModelBadMigrations,
    DerivedModel,
    Derived2dModel,
};

lazy_static!{
    static ref DB: Database = {
        let host = env::var("HOST").expect("Environment variable HOST must be defined.");
        let port = env::var("PORT").expect("Environment variable PORT must be defined.")
            .parse::<u32>().expect("Environment variable PORT must be an instance of `u32`.");
        let connection_string = format!("mongodb://{}:{:?}/", host, port);
        Client::with_uri(&connection_string)
            .expect("Expected MongoDB instance to be available for testing.")
            .db("witherTestDB")
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
        // Delete any records in the collection for respective models.
        User::delete_many(DB.clone(), doc!{}).expect("Expected to successfully delete all records for test fixture.");
        UserModelBadMigrations::delete_many(DB.clone(), doc!{}).expect("Expected to successfully delete all records for test fixture.");
        DerivedModel::delete_many(DB.clone(), doc!{}).expect("Expected to successfully delete all records for test fixture.");

        // Clean up any indices.
        let users_coll = DB.clone().collection(User::COLLECTION_NAME);
        for idx in User::indexes().into_iter() {
            let _ = (&users_coll).drop_index_model(idx);
        }
        let other_users_coll = DB.clone().collection(UserModelBadMigrations::COLLECTION_NAME);
        for idx in UserModelBadMigrations::indexes().into_iter() {
            let _ = (&other_users_coll).drop_index_model(idx);
        }
        let derivations_coll = DB.clone().collection(DerivedModel::COLLECTION_NAME);
        for idx in DerivedModel::indexes().into_iter() {
            let _ = (&derivations_coll).drop_index_model(idx);
        }
        let derivations2d_coll = DB.clone().collection(Derived2dModel::COLLECTION_NAME);
        for idx in Derived2dModel::indexes().into_iter() {
            let _ = (&derivations2d_coll).drop_index_model(idx);
        }
        self
    }

    /// Drop the database which is used by this harness.
    pub fn with_dropped_database(self) -> Self {
        DB.clone().drop_database().expect("Expected to be able to drop database.");
        self
    }

    /// Sync all of the data models used by this harness.
    pub fn with_synced_models(self) -> Self {
        User::sync(DB.clone()).expect("Expected to successfully sync `User` model.");
        // UserModelBadMigrations::sync(DB.clone()).expect("Expected to successfully sync `UserModelBadMigrations` model.");
        DerivedModel::sync(DB.clone()).expect("Expected to successfully sync `DerivedModel` model.");
        Derived2dModel::sync(DB.clone()).expect("Expected to successfully sync `Derived2dModel` model.");
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
