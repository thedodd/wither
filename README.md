wither
======
[![Build Status](https://travis-ci.org/thedodd/wither.svg?branch=master)](https://travis-ci.org/thedodd/wither)
[![](https://img.shields.io/badge/tested%20on-mongodb%203.2%2B-brightgreen.svg)](#)
[![Crates.io](https://img.shields.io/crates/v/wither.svg)](https://crates.io/crates/wither)
[![docs.rs](https://docs.rs/wither/badge.svg)](https://docs.rs/wither)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

An ODM for MongoDB built upon the [mongo rust driver](https://github.com/mongodb-labs/mongo-rust-driver-prototype). Please ⭐ on [github](https://github.com/thedodd/wither)!

A primary goal of this project is to provide a simple, sane & predictable interface into MongoDB based on data models. If at any point this system might get in your way, you have direct access to the underlying driver. This project is tested against MongoDB `3.2`, `3.4`, `3.6` & `4.0`.

Migrations have landed! Check out the docs for more info on getting started with migrations.

This project makes use of `associated constants` as of `0.2.0`, so you will need to be running rust `>= 1.20`.

**NOTE:** progress is being, but there is a lot more to be done! For the time being, there may be backwards incompatible releases made from minor version to minor version until the best patterns for this library are found. It would be best to pin to an exact version in your `Cargo.toml`. Any such backwards incompatible changes will be declared in the [changelog](https://github.com/thedodd/wither/master/wither/CHANGELOG.md).

Check out the [changelog](https://github.com/thedodd/wither/master/wither/CHANGELOG.md) for more details on what has happened from release to release.

### current plans & direction
Progress is being made towards a `0.6.0` release. This will be a backwards incompatible release ... but you should be able to remove a decent bit of code. The release will be focused on implementing a custom derive for defining your models, which is 95% of what it takes to get started with this crate. Seems like a big win, IMHO.

### getting started
A minimal example of how you might define a model for a MongoDB collection.

```rust
#[macro_use]
extern crate bson;
extern crate serde;
#[macro_use(Serialize, Deserialize)]
extern crate serde_derive;
extern crate wither;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl<'a> wither::Model<'a> for User {

    /// The name of this model's collection.
    const COLLECTION_NAME: &'static str = "users";

    /// Implement the getter for the ID of a model instance.
    fn id(&self) -> Option<bson::oid::ObjectId> {
        return self.id.clone();
    }

    /// Implement the setter for the ID of a model instance.
    fn set_id(&mut self, oid: bson::oid::ObjectId) {
        self.id = Some(oid);
    }
}
```

#### logging
This create uses the [rust standard logging facade](https://docs.rs/log/), and integrating it with another logging framework is usually quite simple. If you are using slog, check out the [slog-rs/stdlog](https://docs.rs/slog-stdlog/) create for easy integration.

#### next steps
Now you are ready to tackle some of the other important parts of the model lifecycle. Some additional items to look into:

- [Model::sync](https://docs.rs/wither/0.5.2/wither/model/index.html#sync) - sync your models with the backend.
- [Model::indexes](https://docs.rs/wither/0.5.2/wither/model/index.html#indexes) - define indexes for your models.
- [Model::migrations](https://docs.rs/wither/0.5.2/wither/model/index.html#migrations) - define migrations to be run against your models collections.

Good luck on the path.

### development
#### tests
For easy backend integration, this project is using docker compose to manage MongoDB instances. Before kicking off tests locally, simply execute `docker-compose up -d` to bring up all of the different MongoDB instances.

Now you just need to invoke the tests with the appropriate environment variables exposed:

```bash
# Test against MongoDB 3.2.
HOST=localhost PORT=27017 cargo test -p wither --tests --lib -- --test-threads=1

# Test against MongoDB 3.4.
HOST=localhost PORT=27117 cargo test -p wither --tests --lib -- --test-threads=1

# Test against MongoDB 3.6.
HOST=localhost PORT=27217 cargo test -p wither --tests --lib -- --test-threads=1

# Test against MongoDB 4.0.
HOST=localhost PORT=27317 cargo test -p wither --tests --lib -- --test-threads=1
```
