changelog
=========

## [unreleased]

### added
- Added new `ModelCursor` type which wraps a cursor and yields model instances.

### changed
- Cut all crates over to edition 2018.
- Updated lots of deps. Importantly:
    - `bson@0.14` and is now a public export of this crate.
    - `mongodb@0.9` and is now a public export of this crate.
- Updated `Model::find` to match new driver's interface, and returns a `ModelCursor` wrapping the standard cursor.
- Updated `Model::model_write_concern` to `Model::write_concern` and updated defaults to match driver defaults.
- Models are now constrained by `DeserializeOwned` now, instead of `Deserialize`.

### removed

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
