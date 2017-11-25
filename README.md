wither
======
[![Build Status](https://travis-ci.org/thedodd/wither.svg?branch=master)](https://travis-ci.org/thedodd/wither)
[![](https://img.shields.io/badge/tested%20on-mongodb%203.2%2B-brightgreen.svg)](#)
[![Crates.io](https://img.shields.io/crates/v/wither.svg)](https://crates.io/crates/wither)
[![docs.rs](https://docs.rs/wither/badge.svg)](https://docs.rs/wither)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

An ODM for MongoDB built upon the [mongo rust driver](https://github.com/mongodb-labs/mongo-rust-driver-prototype). Please ⭐ on [github](https://github.com/thedodd/wither)!

A primary goal of this project is to provide a simple, sane & predictable interface into MongoDB based on data models. If at any point this system might get in your way, you have direct access to the underlying driver.

This project makes use of `associated constants` as of `0.2.0`, so you will need to be running rust `>= 1.20`.

**NOTE:** progress is being, but there is a lot more to be done! For the time being, there may be backwards incompatible releases made from minor version to minor version until the best patterns for this library are found. It would be best to pin to an exact version in your `Cargo.toml`. Any such backwords incompatible changes will be declared in the [changelog](./CHANGELOG.md).

Check out the [changelog](./CHANGELOG.md) for more details on what has happened from release to release.

### example
An example of how you might use this library to define a model for a MongoDB collection.

```rust
#[macro_use]
extern crate bson;
extern crate chrono;
extern crate mongodb;
extern crate serde;
#[macro_use(Serialize, Deserialize)]
extern crate serde_derive;
extern crate wither;

use mongodb::coll::options::IndexModel;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl<'a> wither::Model<'a> for User {

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
        return vec![
            Box::new(wither::IntervalMigration{
                name: String::from("remove-oldfield"),
                threshold: chrono::Utc.ymd(2100, 1, 1).and_hms(1, 0, 0),
                filter: doc!{"oldfield": doc!{"$exists": true}},
                set: None,
                unset: Some(doc!{"oldfield": ""}),
            }),
        ];
    }
}

fn main() {
    let client = mongodb::Client::with_uri("mongodb://tests.mongodb:27017/").unwrap();
    let db = client.db("usersService");

    let mut user = User{id: None, email: "test@test.com".to_string()};
    user.save(db.clone(), None).expect("Expected a successful save operation.");
    let user_from_db = User.find_one(db.clone(), Some(doc!{"_id" => (user.id.clone().unwrap())}))
        .expect("Expected a successful lookup.") // Unwraps the Result.
        .expect("Expected a populated value from backend."); // Unwraps the optional model instance.
}
```

### migrations
Please note that `Model`s defined in this system use [serde](https://serde.rs/), and as such, it is quite likely that no explicity schema migration is needed for changes to your model. Often times, [field defaults](https://serde.rs/field-attrs.html#serdedefault) can be used and no additional overhead would be required.

With that said, schema migrations in this system:
- are defined in Rust code. Allowing them to live as child elements of your data models.
- are executed per model, whenever `Model::sync` is called — which should be once per system life cycle, early on at boottime. When dealing with an API service, this should occur before the API begins handling traffic.
- require no downtime to perform.
- require minimal configuration. The logic you use directly in your model for connecting to the backend is used for the migrations system as well.
- require no imperative logic. Simply declare your `filter`, `$set` & `$unset` documents, and the rest will be taken care of.

An important question which you should be asking at this point is _"Well, how is this going to work at scale?"._ This is an excellent question, of course. The answer is that it depends on how you write your migrations. Here are a few pointers & a few notes to help you succeed.
- be sure that the queries used by your migrations are covered. Just add some new indexes to your `Model::indexes` implementation to be sure. Indexes will always be synced by `Model::sync` before migrations are executed for this reason.
- when you are dealing with massive amounts of data, and every document needs to be touched, **indexing still matters!** Especially when using an `IntervalMigration`, as you may be under heavy write load, and new documents will potentially be introduced having the old schema after the first service performs the migration. Schema convergence will only take place after all service instances have been updated & have executed their migrations.

##### IntervalMigration
This migration type is based on a time window. The migration will be executed every time `Model::sync` is called, until the migration's time `threshold` is passed, which will cause the migration to no-op.
