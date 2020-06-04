<h1 align="center">wither</h1>
<div align="center">
    <strong>
An ODM for MongoDB built upon the official <a href="https://github.com/mongodb/mongo-rust-driver">MongoDB Rust driver</a>. Please ⭐ on <a href="https://github.com/thedodd/wither">github</a>!
    </strong>
</div>
<br />
<div align="center">

[![Build Status](https://travis-ci.org/thedodd/wither.svg?branch=master)](https://travis-ci.org/thedodd/wither)
[![](https://img.shields.io/badge/tested%20on-mongodb%203.6%2B-brightgreen.svg)](#)
[![Crates.io](https://img.shields.io/crates/v/wither.svg)](https://crates.io/crates/wither)
[![docs.rs](https://docs.rs/wither/badge.svg)](https://docs.rs/wither)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
![Crates.io](https://img.shields.io/crates/d/wither.svg)

</div>

The primary goal of this project is to provide a simple, sane & predictable interface into MongoDB based on data models. If at any point this system might get in your way, you have direct access to the underlying driver. This project is tested against MongoDB `3.6`, `4.0` & `4.2`.

**GREAT NEWS!** We've finally updated to the officially supported [Rust driver](https://github.com/mongodb/mongo-rust-driver). **Even better news!** The driver team has been hard at work, and has implemented support for async IO in `mongodb@0.10.x` and on. Our next major focus with the Wither project is to make it compatible with the async driver.

Due to updates in the underlying driver, there is a fair number of breaking changes in the `Model` trait, as well as the `Model` derive macro. Details can be found in the changelog and the documentation.

### items of interest
- [docs](https://docs.rs/wither): all the good stuff is here.
- [changelog](https://github.com/thedodd/wither/blob/master/CHANGELOG.md): details on what has happened from release to release.
- [contributing & development guidelines](https://github.com/thedodd/wither/blob/master/CONTRIBUTING.md): details on how to get started with doing development on this project.

### getting started
To get started, simply derive `Model` on your struct along with a few other serde derivations. Let's step through a full example.

```rust ,no_run
use serde::{Serialize, Deserialize};
use wither::prelude::*;
use wither::bson::{doc, oid::ObjectId};
use wither::mongodb::{Client, error::Result};

// Now we define our model. Simple as deriving a few traits.
#[derive(Debug, Model, Serialize, Deserialize)]
#[model(index(keys=r#"doc!{"email": 1}"#, options=r#"doc!{"unique": true}"#))]
struct User {
    /// The ID of the model.
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,
    /// This field has a unique index on it.
    pub email: String,
}

fn main() -> Result<()> {
    // Connect & sync indexes.
    let db = Client::with_uri_str("mongodb://localhost:27417/")?.database("mydb");
    User::sync(db.clone())?;

    // Create a user.
    let mut me = User{id: None, email: String::from("my.email@example.com")};
    me.save(db.clone(), None)?;

    // Update user's email address.
    me.update(db.clone(), None, doc!{"$set": doc!{"email": "new.email@example.com"}}, None)?;

    // Fetch all users.
    for user in User::find(db.clone(), None, None)? {
        println!("{:?}", user);
    }
    Ok(())
}

```

#### next steps
And that's all there is to it. Now you are ready to tackle some of the other important parts of the model lifecycle. Some additional items to look into:

- [deriving model](https://docs.rs/wither/latest/wither/model/trait.Model.html) - learn more about automatically deriving the `Model` trait on your structs.
- [model usage](https://docs.rs/wither/latest/wither/model/trait.Model.html#provided-methods) - check out some of the other methods available to you from your models.
- [syncing indexes](https://docs.rs/wither/latest/wither/model/trait.Model.html#sync) - learn how to synchronize a model's indexes with the database.
- [logging](https://docs.rs/wither/latest/wither/model/trait.Model.html#logging) - learn how to hook into this crate's logging mechanisms.
- [migrations](https://docs.rs/wither/latest/wither/migration/index.html) - learn about defining migrations to be run against your model's collection.

Good luck on the path.
