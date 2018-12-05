Derive the `Model` trait on your data model structs.

All `Model` struct & field attributes are declared inside of `model(...)` attributes as such: `#[model(<attrs>)]`.

### derive
Deriving `Model` for your struct is straightforward.

- Ensure that your struct has at least the following derivations: `#[derive(Model, Serialize, Deserialize)]`.
- Ensure that you have a field named `id`, of type `Option<ObjectId>`, with the following serde attributes: `#[serde(rename="_id", skip_serializing_if="Option::is_none")]`.

For now, it seems logical to disallow customization of the PK. An argument could be made for allowing full customization of the PK for a MongoDB collection, but there really is no end-all reasoning for this argument which I am aware of. If you need to treat a different field as PK, then just add the needed index to the field, and you are good to go. More on indexing soon.

If you need to implement `Serialize` and/or `Deserialize` manually, add the `#[model(skip_serde_checks)]` struct attribute, then you may remove the respective derivations mentioned above. If you are handling the `id` field manually as well, then you may remove the `rename` & `skip_serializing_if` attributes as well. However, take care to ensure that you are replicating the serde behavior of these two attributes, else you may run into strange behavior ... and it won't be my fault `;p`.

### struct attributes
There are a few struct-level `Model` attributes available currently.

- `collection_name="..."`: this allows you to specify your model name explicitly. By default, your model's name will be pluralized, and then formatted as a standard table name (with underscores, EG `OrgPermission` becomes `org_permissions`).
- `skip_serde_checks`: including this attribute will disable any checks which are normally performed to ensure that serde is setup properly on your model. If you disable serde checks, you're on your own `:)`.
- `wc_replication="..."`: this controls the model's write concern replication setting. Must be an `i32` wrapped in a string. Defaults to `1`. It is acknowledged that it is possible to specify `"majority"` in MongoDB itself, but the underlying MongoDB driver does not currently support that option. If this is an issue for you, please open an issue in this repo.
- `wc_timeout="..."`: this controls the model's write concern replication timeout setting. Must be an `i32` wrapped in a string. Defaults to `0`.
- `wc_journaling="..."`: this controls the model's write concern replication journaling setting. Must be a `bool` wrapped in a string. Defaults to `true`.
- `wc_fsync="..."`: this controls the model's write concern replication fsync setting. Must be a `bool` wrapped in a string. Defaults to `false`.

### indexing
Adding indexes to your model's collection is done entirely through the attribute system. Let's start with an example.

```rust
# #[macro_use]
# extern crate mongodb;
# extern crate serde;
# #[macro_use(Serialize, Deserialize)]
# extern crate serde_derive;
# extern crate wither;
# #[macro_use(Model)]
# extern crate wither_derive;
#
# use wither::prelude::*;
# use mongodb::coll::options::IndexModel;
#
/// An example model.
#[derive(Model, Serialize, Deserialize)]
struct MyModel {
    // snip ...
    # #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    # pub id: Option<mongodb::oid::ObjectId>,

    /// This field has a unique index on it.
    #[model(index(index="dsc", unique="true"))]
    pub email: String,

    /// First field of a compound index.
    #[model(index(index="dsc", with(field="last_name", index="dsc")))]
    pub first_name: String,

    /// Is indexed along with `first_name`, but nothing special is declared here.
    pub last_name: String,

    // snip ...
}
#
# fn main() {}
```

As you can see, everything is declared within `#[model(index(...))]` attributes. Let's break this down.

##### index
Everything related to an index declaration must be declared within these parens. The initial field of the index will be the field which this declaration appears on. If the field is using a serde `rename` attribute, this system will use the value of `rename` as the initial field name for the new index. For indexing embedded documents, see the [embedded index](#embedded-index) section.

##### index type
Use `index="TYPE"` to declare the type of index for the field which this attribute appears on. The value must be one of the valid MongoDB index types:  `"asc"`, `"dsc"`, `"2d"`, `"2dsphere"`, `"geoHaystack"`, `"text"` & `"hashed"`.

##### embedded index
If you need to index an embedded document underneath the field which the index declaration appears on, use the attribute `embedded="subdoc1.subdoc2.subdocN.target_field"`. Use dot notation to traverse to more deeply embedded documents. The value provided to `embedded` will be concatenated with the field name of the field which this index declaration appears on. If the root field has a serde `rename` attribute, it will be accounted for. Embedded structs which have serde `rename` attributes will not be accounted for by this system, you must specify the name correctly, as it will be in the database, when declaring the `embedded="..."` value.

Consider the following example.

```rust
# #[macro_use]
# extern crate mongodb;
# extern crate serde;
# #[macro_use(Serialize, Deserialize)]
# extern crate serde_derive;
# extern crate wither;
# #[macro_use(Model)]
# extern crate wither_derive;
#
# use wither::prelude::*;
# use mongodb::coll::options::IndexModel;
use mongodb::Document;

#[derive(Model, Serialize, Deserialize)]
struct MyModel {
    # #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    # pub id: Option<mongodb::oid::ObjectId>,
    // ... other model fields

    /// A subdocument which may be another struct or a BSON Document.
    #[model(index(index="asc", embedded="targetField"))]
    pub metadata: Document,
}
# fn main() {}
```

In the above example, an `ascending` index will be generated on `metadata.targetField`.

##### with
This is optional. For compound indexes, this is where you declare the other fields which the generated index is to be created with. Inside of these parens, you must provide exactly two key-value pairs as follows: `with(field="...", index="...")`. Where `field="..."` is the field to include in the index. You may use dot notation for indexing embedded documents. The value for `index="..."` must be one of the valid index types, as mentioned in the [index types](#index-type) section. Declare an additional `with` token for each additional field which needs to be a part of this index.

In versions `0.6 — 0.7`, this attribute took key-value pairs corresponding to field names and index types. As keys in Rust’s [syn::MetaNameValue](https://docs.rs/syn/latest/syn/struct.MetaNameValue.html) system are not allowed to contain the character `.`, this pattern was inadequate for indexing embedded documents. That is the main reason for the current implementation of this attribute.

##### weight
This is optional. When using `text` indexes, you may declare the weights of specific fields of the index using the syntax `weight(field="...", weight="...")`, where `field` is the field name (may be an embedded document field), and `weight` is an `i32` wrapped in a string.

```rust
# #[macro_use]
# extern crate mongodb;
# extern crate serde;
# #[macro_use(Serialize, Deserialize)]
# extern crate serde_derive;
# extern crate wither;
# #[macro_use(Model)]
# extern crate wither_derive;
#
# use wither::prelude::*;
# use mongodb::coll::options::IndexModel;
#
# #[derive(Model, Serialize, Deserialize)]
# struct MyModel {
    # #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    # pub id: Option<mongodb::oid::ObjectId>,

    // ... other model fields

    /// A text search field, so we add `weight` attrs for index for configuration.
    #[model(index(
        index="text", with(field="text1", index="text"),
        weight(field="text0", weight="10"), weight(field="text1", weight="5"),
    ))]
    pub text0: String,

    /// The other field of our text index. No `model` attributes need to be added here.
    pub text1: String,

    // ... other model fields
# }
# fn main() {}
```

Check out the MongoDB docs on [Control Search Results with Weights](https://docs.mongodb.com/manual/tutorial/control-results-of-text-search/) for some excellent guidance on how to effectively use text indexes.

##### other attributes
Other attributes, like `unique` or `sparse`, are optional. Simply use the name of the attribute, followed by `=`, followed by the desired value (which must be quoted). Be sure to comma-separate each attribute-value pair. All attributes supported by the underlying MongoDB driver are supported by this framework. A list of all attributes can be found in the docs for [IndexOptions](https://docs.rs/mongodb/latest/mongodb/coll/options/struct.IndexOptions.html).

##### known issues
- there is a bug in the underlying mongodb driver which is currently making it impossible to declare `geoHaystack` indexes from within this framework. See issue [#23](https://github.com/thedodd/wither/issues/23).
- there is a bug in the underlying mongodb driver which is currently making it impossible to declare an index `storage_engine`, as the type is declared incorrectly. See issue [#22](https://github.com/thedodd/wither/issues/22).
