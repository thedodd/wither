//! wither
//! ======
//! [![Build Status](https://travis-ci.org/thedodd/wither.svg?branch=master)](https://travis-ci.org/thedodd/wither)
//! [![](https://img.shields.io/badge/tested%20on-mongodb%203.2%2B-brightgreen.svg)](#)
//! [![Crates.io](https://img.shields.io/crates/v/wither.svg)](https://crates.io/crates/wither)
//! [![docs.rs](https://docs.rs/wither/badge.svg)](https://docs.rs/wither)
//! [![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
//!
//! An ODM for MongoDB built upon the [mongo rust driver](https://github.com/mongodb-labs/mongo-rust-driver-prototype). Please ⭐ on [github](https://github.com/thedodd/wither)!
//!
//! A primary goal of this project is to provide a simple, sane & predictable interface into MongoDB based on data models. If at any point this system might get in your way, you have direct access to the underlying driver. This project is tested against MongoDB `3.2`, `3.4`, `3.6` & `4.0`.
//!
//! Check out the [changelog](https://github.com/thedodd/wither/master/wither/CHANGELOG.md) for more details on what has happened from release to release.
//!
//! ### getting started
//! A minimal example of how you might define a model for a MongoDB collection & use it.
//!
//! ```rust,no_run
//! #[macro_use]
//! extern crate bson;
//! extern crate mongodb;
//! extern crate serde;
//! #[macro_use(Serialize, Deserialize)]
//! extern crate serde_derive;
//! extern crate wither;
//! #[macro_use(Model)]
//! extern crate wither_derive;
//!
//! use mongodb::{
//!     Client,
//!     db::{Database, ThreadedDatabase},
//!     coll::options::IndexModel,
//!     ThreadedClient,
//! };
//! use wither::prelude::*;
//!
//! /// An example model.
//! #[derive(Model, Serialize, Deserialize)]
//! struct User {
//!     /// The ID of the model.
//!     #[serde(rename="_id", skip_serializing_if="Option::is_none")]
//!     pub id: Option<bson::oid::ObjectId>,
//!
//!     /// This field has a unique index on it.
//!     #[model(index(index_type="dsc", unique="true"))]
//!     pub email: String,
//! }
//!
//! fn main() {
//!     // Connect to your database as you normally would with the `mongodb` crate.
//!     let db = Client::with_uri("mongodb://host:port/").unwrap().db("mydb");
//!
//!     // Sync indexes to the database.
//!     User::sync(db.clone()).unwrap();
//!
//!     // Fetch all users.
//!     let all_users = User::find(db.clone(), None, None).unwrap();
//!
//!     // Find a specific user. Returns `Result<Option<Self>>`.
//!     let mut me = User::find_one(db.clone(), Some(doc!{"email": "my.email@example.com"}), None).unwrap().unwrap();
//!
//!     // Update your email address.
//!     me = me.update(db.clone(), doc!{"$set": doc!{"email": "new.email@example.com"}}, None).unwrap();
//!
//!     println!("{}", me.email); //>>> new.email@example.com
//! }
//! ```
//!
//! #### logging
//! This create uses the [rust standard logging facade](https://docs.rs/log/), and integrating it with another logging framework is usually quite simple. If you are using slog, check out the [slog-rs/stdlog](https://docs.rs/slog-stdlog/) create for easy integration.
//!
//! #### next steps
//! Now you are ready to tackle some of the other important parts of the model lifecycle. Some additional items to look into:
//!
//! - [Model::migrations](https://docs.rs/wither/latest/wither/model/index.html#migrations) - define migrations to be run against your model's collection.
//!
//! Good luck on the path.

#[macro_use(doc, bson)]
pub extern crate bson;
extern crate chrono;
#[macro_use]
extern crate log;
pub extern crate mongodb;
extern crate serde;

pub mod migration;
pub mod model;

// Expose lower symbols in the top level module.
pub use migration::{
    IntervalMigration,
    Migration,
};
pub use model::{
    basic_index_options,
    Model,
};

pub mod prelude {
    pub use ::migration::Migration;
    pub use ::model::Model;
}
