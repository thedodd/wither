Derive the `Model` trait on your data model structs.

All `Model` struct & field attributes are declared inside of `model(...)` attributes as such: `#[model(<attrs>)]`.

### derive
Deriving `Model` for your struct is straightforward.

- Ensure that your struct has at least the following derivations: `#[derive(Model, Serialize, Deserialize)]`.
- Ensure that you have a field named `id`, of type `Option<bson::oid::ObjectId>`, with the following serde attributes: `#[serde(rename="_id", skip_serializing_if="Option::is_none")]`.

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
    #[model(index(index_type="dsc", unique="true"))]
    pub email: String,

    /// First field of a compound index.
    #[model(index(index_type="dsc", with(last_name="dsc")))]
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
Everything related to an index declaration must be declared within these parens. If the field is using a serde `rename` attribute, this system will account for that and use the value of `rename` as the initial field name for the new index.

##### type
This declares the type of index for the field which this attribute appears on, which will also be the first field of the generated index. The value must be one of the valid MongoDB index types:  `"asc"`, `"dsc"`, `"2d"`, `"2dsphere"`, `"geoHaystack"`, `"text"` & `"hashed"`.

##### with
This is optional. For compound indexes, this is where you declare the other fields which the generated index is to be created with. Inside of these parens, you map field names to index types. The field name must be the name of the target field as it will be in the MongoDB collection. The value must be one of the valid MongoDB index types, as described above.

##### weights
This is optional. Values here simply map field names to `i32` values wrapped in strings.

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

    // A text search field, so we add a `weights` field on our index for optimization.
    #[model(index(index_type="text", with(text1="text"), weights(text0="10", text1="5")))]
    pub text0: String,

    // The other field of our text index. No `model` attributes need to be added here.
    pub text1: String,

    // ... other model fields
# }
# fn main() {}
```

Check out the MongoDB docs on [Control Search Results with Weights](https://docs.mongodb.com/manual/tutorial/control-results-of-text-search/) for some excellent guidance on how to effectively use text indexes.

##### other attributes
Other attributes, like `unique` or `sparse`, are optional. Simply use the name of the attribute, followed by `=`, followed by the desired value (which must be quoted). Be sure to comma-separate each attribute-value pair. All attributes supported by the underlying MongoDB driver are supported by this framework. A list of all attributes can be found in the docs for [IndexOptions](https://docs.rs/mongodb/latest/mongodb/coll/options/struct.IndexOptions.html).

##### known issues
- As of the `0.6.0` implementation, specifying the additional fields of a compound index using this system my be theoretically limiting. Technically, the field names are declared as Rust `syn::Ident`s, which carries the restriction of being a valid variable name, which is more limiting than that of MongoDB's field naming restrictions. **Please open an issue** if you find this to be limiting. There are workarounds, but if this is a big issue, I definitely want to know. There are other ways this could be implemented.
- Indexing subdocuments is in progress, but not done yet. Will probably come as `0.7` or something.
- To index a field on a subdocument which is not modelled (EG, using `Document` as a value for a field), you will have to manually implement `Model` for your struct & then manually specify the indexes. See the section on [manually implementing model](#manually-implementing-model).
