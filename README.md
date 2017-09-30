wither
======
[![Build Status](https://travis-ci.org/thedodd/wither.svg?branch=master)](https://travis-ci.org/thedodd/wither)
[![Crates.io](https://img.shields.io/crates/v/wither.svg)](https://crates.io/crates/wither)
[![docs.rs](https://docs.rs/wither/badge.svg)](https://docs.rs/wither)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

An ODM for MongoDB built upon the [mongo rust driver](https://github.com/mongodb-labs/mongo-rust-driver-prototype).

This project makes use of `associated constants` as of `0.2.0`, so you will need to be running rust `>= 1.20`.

**NOTE:** this project is fairly nascent and is being built out as I continue to use it in my own projects. For the time being, there may be backwards incompatible releases made from minor version to minor version until the best patterns for this library are found. It would be best to pin to an exact version in your `Cargo.toml`.

Check out the [changelog](./CHANGELOG.md).

### example
An example of how you might use this library to define a model for a MongoDB collection.

```rust
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
