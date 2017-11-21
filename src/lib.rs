//! An ODM for MongoDB built upon the [mongo rust driver](https://github.com/mongodb-labs/mongo-rust-driver-prototype). Please ⭐ on [github](https://github.com/thedodd/wither)!
//!
//! A primary goal of this project is to provide a simple, sane & predictable interface into MongoDB based on data models. If at any point this system might get in your way, you have direct access to the underlying driver.
//!
//! This project makes use of `associated constants` as of `0.2.0`, so you will need to be running rust `>= 1.20`.
//!
//! An example of how you might use this library to define a model for a MongoDB collection.
//!
//! ```rust,ignore
//! #[derive(Serialize, Deserialize, Debug, Clone)]
//! pub struct User {
//!     /// The user's unique ID.
//!     #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
//!     pub id: Option<bson::oid::ObjectId>,
//!
//!     /// The user's unique email.
//!     pub email: String,
//! }
//!
//! impl<'a> wither::Model<'a> for User {
//!
//!     const COLLECTION_NAME: &'static str = "users";
//!
//!     fn id(&self) -> Option<bson::oid::ObjectId> {
//!         return self.id.clone();
//!     }
//!
//!     fn set_id(&mut self, oid: bson::oid::ObjectId) {
//!         self.id = Some(oid);
//!     }
//!
//!     fn indexes() -> Vec<IndexModel> {
//!         return vec![
//!             IndexModel{
//!                 keys: doc!{"email" => 1},
//!                 options: wither::basic_index_options("unique-email", true, Some(true), None, None),
//!             },
//!         ];
//!     }
//! }
//! ```

#[macro_use(doc, bson)]
extern crate bson;
extern crate chrono;
#[macro_use]
extern crate log;
extern crate mongodb;
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
