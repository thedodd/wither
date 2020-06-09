As your system evolves over time, you may find yourself needing to alter the data in your databases in a way which does not fit in with the standard `Model` lifecycle. Using the migration system will keep your database clean, and will allow you to evolve your data at a more rapid and controlled pace.

Migrations are controlled by implementing the [Migrating](./trait.Migrating.html) trait on your `Model`s. This couldn't be more simple, so let's look at an example of an [`IntervalMigration`](./struct.IntervalMigration.html).

```rust
# use chrono::prelude::*;
# use futures::stream::StreamExt;
# use serde::{Serialize, Deserialize};
# use wither::{prelude::*, Result};
# use wither::bson::{doc, oid::ObjectId};
# use wither::mongodb::Client;
#
# #[derive(Debug, Model, Serialize, Deserialize)]
# struct User {
#     #[serde(rename="_id", skip_serializing_if="Option::is_none")]
#     pub id: Option<ObjectId>,
# }
#
impl Migrating for User {
    // Define any migrations which your model needs in this method.
    // As this is an interval migration, it will deactivate itself after the given threshold
    // date, so you could leave it in your code for as long as you would like.
    fn migrations() -> Vec<Box<dyn wither::Migration>> {
        vec![
            Box::new(wither::IntervalMigration{
                name: "remove-oldfield".to_string(),
                // NOTE: use a logical time here. A day after your deployment date, or the like.
                threshold: chrono::Utc.ymd(2100, 1, 1).and_hms(1, 0, 0),
                filter: doc!{"oldfield": doc!{"$exists": true}},
                set: None,
                unset: Some(doc!{"oldfield": ""}),
            }),
        ]
    }
}
```

Then execute `User::migrate` to run the migration.

**Remember, MongoDB is not a SQL based system.** There is no true database level schema enforcement. `IntervalMigration`s bridge this gap quite nicely.

`Model`s defined in this system use [serde](https://serde.rs/), and as such, it is quite likely that no explicit schema migration is needed for changes to your model. Often times, [field defaults](https://serde.rs/field-attrs.html#serdedefault) can be used and no additional overhead would be required. However, when needing to remove fields, change a field type, or manage other aspects of your schema programmatically, migrations can save the day.

With that said, schema migrations in this system:

- are defined in Rust code. Allowing them to live as child elements of your data models.
- are executed per model, whenever [`Migrating::migrate`](../migration/trait.Migrating.html#method.migrate) is called — which should be once per system life cycle, early on at boottime, after indexes have been synced. When dealing with an API service, this should occur before the API begins handling traffic.
- require no downtime to perform.
- require minimal configuration. The logic you use directly in your model for connecting to the backend is used for the migrations system as well.
- require no imperative logic. Simply declare your `filter`, `$set` & `$unset` documents, and the rest will be taken care of.

An important question which you should be asking at this point is _"Well, how is this going to work at scale?"._ The answer is that it depends on how you write your migrations. Here are a few pointers & a few notes to help you succeed.

- be sure that the queries used by your migrations are covered. You can always add new indexes to your `Model` to be sure. Indexes should always be synced first.
- when you are dealing with massive amounts of data, and every document needs to be touched, **indexing still matters!** Especially when using an `IntervalMigration`, as you may be under heavy write load, and new documents will potentially be introduced having the old schema after the first service performs the migration. Schema convergence will only take place after all service instances have been updated & have executed their migrations.

Currently, the following migration types are available.

- [IntervalMigration](./struct.IntervalMigration.html)

If there is a new migration "type" which you find yourself in need of, [please open an issue](https://github.com/thedodd/wither) describing what you need, and we will see what we can put together!
