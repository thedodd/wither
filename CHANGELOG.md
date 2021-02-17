changelog
=========

## [unreleased]

## 0.9.0
Now that `Model::sync` index management is back, we are finally ready to release an official `0.9.0`. All of the changes released so far as part of the various `0.9.0-alpha.*` releases are included as part of this release. Big shoutout to @simoneromano96 for all of their hard work on the updates to `Model::sync`, thank you!

### added
- I am happy to announce that index management via `Model::sync` is back! This has taken much longer than intended, but it is finally back. It also has a few improvements where index options are diffed in order to ensure that subtle updates to an index are observed and indexes are re-created as needed. This closes [#51](https://github.com/thedodd/wither/issues/51).
- Added the `delete_many` method.

## 0.9.0-alpha.2
### changed
- All Model trait methods have been updated to take a reference to a `mongodb::Database` instance, no more `db.clone()` required. Thanks to @mehmetsefabalik for pointing out that this is now possible with the 1.x version of the driver.

## 0.9.0-alpha.1

### added
- Adding `async_trait::async_trait` macro as a pub export of the `wither` crate, so that code generated from `wither_derive` can use `#[wither::async_trait]` instead of requiring users to declare `async-trait` as a dependency. This _seems_ like the right thing to do here ... we'll see. If this causes problems for you (which it shouldn't), please open an issue.

## 0.9.0-alpha.0

### added
- Wither is now based on the official [MongoDB Rust driver](https://github.com/mongodb/mongo-rust-driver).
- Everything is now async. All model methods are now `async`. All synchronous code has been disabled, and will be removed from the tree soon.
- `Model::read_concern`, `Model::write_concern` & `Model::selection_criteria` match the driver's updated mechanism for specifying these values. They may all be derived, and are always used via `Model::collection` and any of the model methods which interact with a model's collection.
- Added new `ModelCursor` type which wraps a cursor and yields model instances.

### changed
- All crates cut over to edition 2018.
- Updated lots of deps. Importantly:
    - `mongodb@1.0` and is now a public export of this crate.
    - `bson@1.0` and is now a public export of this crate.
- Models are now constrained by `DeserializeOwned`, instead of `Deserialize`. This simplifies some aspects of the `Model` trait.
- Nearly all `Model` methods which interact with the collection have been updated to match the driver's new collection interface. Lots of good stuff here. Check out the docs.

#### index management
**UPDATE:** index management is back as of 0.9.0!

Index management has not yet been implemented in the mongodb driver as of `1.0`, and thus the index syncing features of `Model::sync` have been temporarily disabled. The hope is that the mongodb team will be able to land their index management code in the driver soon, at which point we will re-enable the `Model::sync` functionality.

If this is important to you, please head over to [wither#51](https://github.com/thedodd/wither/issues/51) and let us know!

Notwithstanding, there are a few other changes to note:
- we have introduced the `wither::IndexModel` struct as a placeholder. This allows our `Model` interface to stay mostly the same, and allows us to preserve our index derivations using the `#[derive(Model)]` system.
- `Model::sync` is still present and callable, but has been temporarily deprecated until we are able to figure out the index management story. I mostly chose to use the deprecation pattern, even if a bit inaccurate, because it guarantees that folks will have a compiler warning letting them know that the functionality has been temporarily disabled.

#### wither_derive
The model derivation system has been GREATLY improved. The entire crate has been refactored. The code should be much more approachable to new folks wanting to contribute, and should be easier to maintain going forward.

- Errors are now reported via `proc-macro-error` and are fully spanned (which means any attribute errors related to the derive system will be highlighted with pin-point accuracy at compile time).
- We are now using `dtolnay`'s excellent [trybuild](https://github.com/dtolnay/trybuild) for testing the compilation of our derive system. Big win here.
- Index derivations have also been greatly simplified. Now, indexes are specified on the model-level (instead of the field-level, as they were previously).
- Indexes are derived using a `keys` field and an `options` field (which is optional). Both are expected to be quoted `doc!{...}` invocations. See the docs for more details.

### removed
Index management `:'(` ... though this should only be temporary.

## 0.8
The core `wither` crate is 100% backwards compatible with this release, but the `Model` trait has received a few additional methods. Namely the `find_one_and_(delete|replace|update)` methods. Came across a use case where I needed them and then realized that I never implemented them. Now they are here. Woot woot!

The `wither_derive` crate has received a few backwards incompatible changes. The motivation behind doing this is detailed in [#21](https://github.com/thedodd/wither/issues/21). The main issue is that we need the derive system to be abstract enough to deal with embedded documents. The backwards incompatible changes are here.
- within `#[model(index())]`, the `index_type` attr has been reduced to simply be `index`. All of the same rules apply as before. This change was made for ergonomic reasons. Less typing. Easier to follow.
- within `#[model(index())]`, the `with(...)` attr has been updated to support subdocuments. The new syntax for this attr is `with(field="...", index="...")`. Supply a `with(...)` attr for each independent field to include.
- within `#[model(index())]`, the `weights(...)` attr has been updated for the same reason as `with`. Now, you need to supply one `weight(field="...", weight="...")` per field weight you are specifying.

The only net-new item being added here is that now, within `#[model(index())]`, you can use the attr `embedded="..."` to cause the index declaration to apply to the embedded document/field specified. [See the docs for more details](https://docs.rs/wither/latest/wither/model/trait.Model.html#embedded-index).

**It is my sincere hope** that this is the last breaking change I will need to make to this crate before promoting this crate to a `1.0.0` release. Let's hope! Please let me know if there are any issues you have with these updates.

#### 0.8.1

- In order to avoid writing `use mongodb::coll::options::IndexModel` explicitly, use the full path of `IndexModel` when sending code back to compiler.

## 0.7
Minimal changes per this release. The main issue being addressed here is [#20](https://github.com/thedodd/wither/issues/20). It is arguable that this is just a bug fix, but the interface for `Model.update` has changed enough with these updates that a minor version increment is merited.

The main thing to observe here is that `Model.update` now takes an optional filter document. If none is given, then this method will create a filter doc and populate it with the model's ID. If a doc is given, then it will have the `_id` key set (potentially overwritten) to the model's ID. Lastly, this method now returns `Result<Option<Self>>` to reflect the fallible nature of the update with a filter.

## 0.6
Wow! So much stuff here. `0.6` is a big step forward for the ergonomics & usability of the wither system. A custom derive has been introduced (`#[derive(Model)]`), and it is now the recommended way to use this system. This should greatly simplify the process of getting started. Overall this has been an awesome experience putting this together and delving into the custom derive system in Rust. Here are a few of the changes to highlight.
- Use `#[derive(Model)]` to turn your struct into a wither `Model`.
- A model's collection name can be controlled via the `#[model(collection_name="...")]` struct attribute, or left off to have a default name generated based on the struct's name.
- Write concern settings are controlled via struct-level `#[model(wc_*)]` attributes.
- Indexes are defined using the `#[model(index(...))]` attribute.
- A `wither::prelude` module has been added for easily pulling in all traits which you may need to use wither effectively.
- Everything has been thoroughly tested, including exhaustive use of `compiletest-rs` for the code generation of the custom derive.

There is really only one breaking change with this release:
- migrations no longer run as part of the `Model::sync` system. They are now encapsulated in their own trait which must be manually implemented on your models if they need migrations. See the docs for more details.

#### 0.6.0 - 0.6.2
Just dealing with some docs.rs build issues. Yanked `0.6.0` & `0.6.1`.

#### 0.6.3
Using mongodb 0.3.12, which now re-exports the bson crate. This means that we no longer have to deal with errors due to conflicting versions of bson.


## 0.5
- a migrations system has been added, closing [#3](https://github.com/thedodd/wither/issues/3) & [#4](https://github.com/thedodd/wither/issues/4). The important part of this feature set is the `IntervalMigration` type.
- `Model::sync` has received some updates. It now synchronizes a model's indexes as well as its migrations.
- reexports of `bson::Document` & `mongodb::coll::options::FindOptions` have been removed. They must be vestiges of my early development on this crate.
- now testing all builds against MongoDB 3.2.x & 3.4.x. Will add 3.6.x when it is available on hub.docker.com.

#### 0.5.1
- updating create dependencies. No expected backwards incompatibilities from this.
- added some notes to the documentation on how to integrate this crate's logging.
- added MongoDB 3.6.x to the test matrix & update patch versions of existing versions in test matrix.

#### 0.5.2
This is an important release which implements a workaround for [a bug reported against the mongodb lib](https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/251). The bug was causing model syncing to fail for new models.
- a workaround was implemented for the above mentioned bug.
- a few `doc!` usages were updated to use `:` as opposed to `=>` for key/val delimiter.

##### backwards incompatibilities
- `Model::sync` no longer panics. It will now return a `Result`, offering users a greater level of control on behavior.

## 0.4
- adds `Model.update`.

## 0.3
- adds `Model::count`.

## 0.2
- adopts usage of `associated constants` for `Model`. Thus, `Model::collection_name()` has been replaced by the associated constant `Model::COLLECTION_NAME` and is required when adopting the `Model` trait for your types.
- adds an actual implementation for `Model::find`.

###### backwards incompatibilities
- `Model::collection_name` has been replaced with `Model::COLLECTION_NAME`.

#### 0.2.1
- no code changes here, just updating some docs & badge links.

## 0.1
Initial release.
