Derive the `Model` trait on your data model structs.

All `Model` attributes are declared inside of `model(...)` attributes as such: `#[model(<attrs>)]`.

### derive
Deriving `Model` for your struct is straightforward.

- Ensure that your struct has at least the following derivations: `#[derive(Model, Serialize, Deserialize)]`.
- Ensure that you have a field named `id`, of type `Option<ObjectId>`, with at least the following serde attributes: `#[serde(rename="_id", skip_serializing_if="Option::is_none")]`.

For now, it seems logical to disallow customization of the PK. An argument could be made for allowing full customization of the PK for a MongoDB collection, but there really is no end-all reasoning for this argument which I am aware of. If you need to treat a different field as PK, then just add the needed index to the field, and you are good to go. More on indexing soon.

If you need to implement `Serialize` and/or `Deserialize` manually, add the `#[model(skip_serde_checks)]` struct attribute, then you may remove the respective derivations mentioned above. If you are handling the `id` field manually as well, then you may remove the `rename` & `skip_serializing_if` attributes as well. However, take care to ensure that you are replicating the serde behavior of these two attributes, else you may run into strange behavior.

### all available model attributes
All attributes available for use when deriving a model are described below:

- `collection_name="..."`: this allows you to specify your model name explicitly. By default, your model's name will be pluralized, and then formatted as a standard table name (with underscores, EG `OrgPermission` becomes `org_permissions`).
- `skip_serde_checks`: including this attribute will disable any checks which are normally performed to ensure that serde is setup properly on your model. If you disable serde checks, you're on your own `:)`.
- `read_concern`: include this attribute to define the read-concern which is to be used when reading data from the model's collection.
- `write_concern`: include this attribute to define the write-concern which is to be used when writing data to the model's collection.
- `selection_criteria`: include this attribute to define the server selection algorithm to use when interacting with the database.
- `index`: include one or more of these attributes to define the set of indexes which should build on the model's collection. **PLEASE NOTE:** as of `0.9.0-alpha.0` index management has been temporarily disabled due to limitations in the underlying driver. We are hoping to get this functionality back soon.

### read concern
To derive this attribute, specify one of the canonical values for `read_concern` recognized by MongoDB. `#[model(read_concern="linearizable")]` will configure the linearizable read concern.

For custom configuration, use this pattern: `#[model(read_concern(custom="custom-concern"))]`. The value of `custom` must be a string literal.

### write_concern
To derive this attribute, specify the write concern options which you would like to configure as so: `#[model(write_concern(w="majority", w_timeout=10, journal=true))]`. Any combination of these options is allow, as long as options are not repeated and as long as one option is specified.

For custom configuration, use this pattern: `#[model(write_concern(w(custom="custom")))]`.

### selection_criteria
To configure a selection criteria, we had to use a slightly more roundabout approach: `#[model(selection_criteria="MyModel::get_selection_criteria")]`. Here, the value of `selection_criteria` must be a valid Rust path to a function which will return your desired selection criteria.

This pattern is used to address some of the complexities with deriving all possible values which may be specified for selection criteria. If this pattern does not work for your use case, please open an issue and let us know.

### indexing
Index derivations have been GREATLY simplified, and future-proofed, as of `wither@0.9.0`. Now, all indexes are specified using the following pattern.

```rust ,no_run
# use serde::{Serialize, Deserialize};
# use wither::{prelude::*, Result};
# use wither::bson::{doc, oid::ObjectId};
# #[derive(Serialize, Deserialize, Model)]
#[model(
    index(keys=r#"doc!{"id": 1}"#),
    index(keys=r#"doc!{"id": -1}"#, options=r#"doc!{"unique": true}"#),
    index(keys=r#"doc!{"some.nested.field": 1}"#),
)]
struct MyModel {
#    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
#    pub id: Option<ObjectId>,
# }
```

This pattern is impervious to any future changes made to the `keys` and `options` documents expected by MongoDB. All values must be quoted, may use `r#` strings (specify any number of `#` symbols after the `r`, followed by `"..."` and a matching number of `#` symbols following the closing quote), and are expected to be `bson::doc!` invocations, providing the compile time BSON validation we all love.

I am personally quite happy with this approach. It is just another example of the core philosophy behind Wither: provide maximum convenience, but don't get in the way.
