wither
======
[![Build Status](https://travis-ci.org/thedodd/wither.svg?branch=master)](https://travis-ci.org/thedodd/wither)
[![](https://img.shields.io/badge/tested%20on-mongodb%203.2%2B-brightgreen.svg)](#)
[![Crates.io](https://img.shields.io/crates/v/wither.svg)](https://crates.io/crates/wither)
[![docs.rs](https://docs.rs/wither/badge.svg)](https://docs.rs/wither)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
![Crates.io](https://img.shields.io/crates/d/wither.svg)
![Crates.io](https://img.shields.io/crates/dv/wither.svg)
[![GitHub issues open](https://img.shields.io/github/issues-raw/thedodd/wither.svg)]()
[![GitHub issues closed](https://img.shields.io/github/issues-closed-raw/thedodd/wither.svg)]()

An ODM for MongoDB built upon the [mongo rust driver](https://github.com/mongodb-labs/mongo-rust-driver-prototype). Please ⭐ on [github](https://github.com/thedodd/wither)!

The primary goal of this project is to provide a simple, sane & predictable interface into MongoDB based on data models. If at any point this system might get in your way, you have direct access to the underlying driver. This project is tested against MongoDB `3.2`, `3.4`, `3.6` & `4.0`.

### items of interest
- [docs](https://docs.rs/wither): all the good stuff is here.
- [changelog](https://github.com/thedodd/wither/blob/master/CHANGELOG.md): details on what has happened from release to release.
- [contributing & development guidelines](https://github.com/thedodd/wither/blob/master/CONTRIBUTING.md): details on how to get started with doing development on this project.

### getting started
To get started, simply derive `Model` on your struct along with a few other serde derivations. Let's step through a full example with imports and all.

```rust ,no_run
// First, we add import statements for the crates that we need.
// In Rust 2018, `extern crate` declarations will no longer be needed.
#[macro_use]
extern crate mongodb;
extern crate serde;
#[macro_use(Serialize, Deserialize)]
extern crate serde_derive;
extern crate wither;
#[macro_use(Model)]
extern crate wither_derive;

// Next we bring a few types into scope for our example.
use mongodb::{
    Client, ThreadedClient,
    db::{Database, ThreadedDatabase},
    coll::options::IndexModel,
    oid::ObjectId,
};
use wither::prelude::*;

// Now we define our model. Simple as deriving a few traits.
#[derive(Model, Serialize, Deserialize)]
struct User {
    /// The ID of the model.
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,

    /// This field has a unique index on it.
    #[model(index(index_type="dsc", unique="true"))]
    pub email: String,
}

fn main() {
    // Create a user.
    let db = mongodb::Client::with_uri("mongodb://localhost:27017/").unwrap().db("mydb");
    let mut me = User{id: None, email: "my.email@example.com".to_string()};
    me.save(db.clone(), None);

    // Update user's email address.
    me = me.update(db.clone(), doc!{"$set": doc!{"email": "new.email@example.com"}}, None).unwrap();

    // Fetch all users.
    let all_users = User::find(db.clone(), None, None).unwrap();
}
```

#### next steps
And that's all there is to it. Now you are ready to tackle some of the other important parts of the model lifecycle. Some additional items to look into:

- [deriving model](https://docs.rs/wither/latest/wither/model/trait.Model.html) - learn more about automatically deriving the `Model` trait on your structs.
- [model usage](https://docs.rs/wither/latest/wither/model/trait.Model.html#provided-methods) - check out some of the other methods available to you from your models.
- [syncing indexes](https://docs.rs/wither/latest/wither/model/trait.Model.html#sync) - learn how to synchronize a model's indexes with the database.
- [logging](https://docs.rs/wither/latest/wither/model/trait.Model.html#logging) - learn how to hook into this crate's logging mechanisms (hint, we use Rust's standard logging facade).
- [migrations](https://docs.rs/wither/latest/wither/migration/index.html) - learn about defining migrations to be run against your model's collection.

Good luck on the path.
