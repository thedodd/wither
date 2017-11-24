extern crate chrono;
// #[macro_use(lazy_static)]
// extern crate lazy_static;
extern crate mongodb;
extern crate wither;

use std::env;

use bson;
use chrono::TimeZone;
use mongodb::coll::options::IndexModel;
use mongodb::db::{Database, ThreadedDatabase};
use mongodb::ThreadedClient;
use wither::Model;

lazy_static!{
    static ref DB: Database = {
        let host = env::var("BACKEND_HOST").expect("Environment variable BACKEND_HOST must be defined.");
        let port = env::var("BACKEND_PORT").expect("Environment variable BACKEND_PORT must be defined.")
            .parse::<u32>().expect("Environment variable BACKEND_PORT must be an instance of `u32`.");
        let connection_string = format!("mongodb://{}:{:?}/", host, port);
        mongodb::Client::with_uri(&connection_string)
            .expect("Expected MongoDB instance to be available for testing.")
            .db("witherTestDB")
    };
}

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
                set: doc!{"testfield": "test"},
                unset: doc!{},
            }),
        ]
    }
}

pub fn setup() -> Database {
    // Delete any records in the collection for respective models.
    User::delete_many(DB.clone(), doc!{}).expect("Expected to successfully delete all records for test fixture.");

    // Clean up any indices.
    let coll = DB.clone().collection(User::COLLECTION_NAME);
    for idx in User::indexes().into_iter() {
        let _ = (&coll).drop_index_model(idx);
    }

    return DB.clone();
}
