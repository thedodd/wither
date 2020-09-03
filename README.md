<h1 align="center">wither</h1>
<div align="center">
    <strong>
An ODM for MongoDB built on the official <a href="https://github.com/mongodb/mongo-rust-driver">MongoDB Rust driver</a>. Please ⭐ on <a href="https://github.com/thedodd/wither">github</a>!
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
</br>

The primary goal of this project is to provide a simple, sane & predictable interface into MongoDB based on data models. If at any point this system might get in your way, you have direct access to the underlying driver. This project is tested against MongoDB `3.6`, `4.0` & `4.2`.

**GREAT NEWS!** Wither is now based on the official [MongoDB Rust driver](https://github.com/mongodb/mongo-rust-driver). Thanks to advancements in the driver, Wither is now fully asynchronous. Simply mirroring the features of the underlying MongoDB driver, Wither supports the following runtimes:
- `tokio-runtime` (default) activates [the tokio runtime](tokio.rs/).
- `async-std-runtime` activates [the async-std runtime](https://async.rs/).

Due to updates in the underlying driver, there is a fair number of breaking changes in the `Model` trait, as well as the `Model` derive macro. Details can be found in the changelog and the documentation. Furthermore, everything is now async by default, and the synchronous interface has been completely removed from the repo.

### items of interest
- [docs](https://docs.rs/wither): all the good stuff is here.
- [changelog](https://github.com/thedodd/wither/blob/master/CHANGELOG.md): details on what has happened from release to release.
- [contributing & development guidelines](https://github.com/thedodd/wither/blob/master/CONTRIBUTING.md): details on how to get started with doing development on this project.

### getting started
To get started, simply derive `Model` on your struct along with a few other serde derivations. Let's step through a full example.

```rust ,no_run
use futures::stream::StreamExt;
use serde::{Serialize, Deserialize};
use wither::{prelude::*, Result};
use wither::bson::{doc, oid::ObjectId};
use wither::mongodb::Client;

// Define a model. Simple as deriving a few traits.
#[derive(Debug, Model, Serialize, Deserialize)]
#[model(index(keys=r#"doc!{"email": 1}"#, options=r#"doc!{"unique": true}"#))]
struct User {
    /// The ID of the model.
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,
    /// The user's email address.
    pub email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Connect & sync indexes.
    let db = Client::with_uri_str("mongodb://localhost:27017/").await?.database("mydb");
    User::sync(&db).await?;

    // Create a user.
    let mut me = User{id: None, email: String::from("my.email@example.com")};
    me.save(&db, None).await?;

    // Update user's email address.
    me.update(&db, None, doc!{"$set": doc!{"email": "new.email@example.com"}}, None).await?;

    // Fetch all users.
    let mut cursor = User::find(&db, None, None).await?;
    while let Some(user) = cursor.next().await {
        println!("{:?}", user);
    }
    Ok(())
}
```

**PLEASE NOTE:** as of the `0.9.0-alpha.0` release, corresponding to the mongodb `1.0` release, index management has not yet been implemented in the mongodb driver, and thus the index syncing features of `Model::sync` have been temporarily disabled. The hope is that the mongodb team will be able to land their index management code in the driver soon, at which point we will re-enable the `Model::sync` functionality.

If this is important to you, please head over to [wither#51](https://github.com/thedodd/wither/issues/51) and let us know!

#### next steps
And that's all there is to it. Now you are ready to tackle some of the other important parts of the model lifecycle. Some additional items to look into:

- [deriving model](https://docs.rs/wither/latest/wither/model/trait.Model.html) - learn more about automatically deriving the `Model` trait on your structs.
- [model usage](https://docs.rs/wither/latest/wither/model/trait.Model.html#provided-methods) - check out some of the other methods available to you from your models.
- [syncing indexes](https://docs.rs/wither/latest/wither/model/trait.Model.html#sync) - learn how to synchronize a model's indexes with the database.
- [logging](https://docs.rs/wither/latest/wither/model/trait.Model.html#logging) - learn how to hook into this crate's logging mechanisms.
- [migrations](https://docs.rs/wither/latest/wither/migration/index.html) - learn about defining migrations to be run against your model's collection.

Good luck on the path.
