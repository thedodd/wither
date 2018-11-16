## manually implementing model
If you need to manually implement the `Model` trait on your data model for some reason, this
section is for you. If not, feel free to skip over it.

### basic impl
```rust
#[macro_use]
extern crate mongodb;
extern crate serde;
#[macro_use(Serialize, Deserialize)]
extern crate serde_derive;
extern crate wither;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::oid::ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl<'a> wither::Model<'a> for User {

    /// The name of this model's collection.
    const COLLECTION_NAME: &'static str = "users";

    /// Implement the getter for the ID of a model instance.
    fn id(&self) -> Option<mongodb::oid::ObjectId> {
        return self.id.clone();
    }

    /// Implement the setter for the ID of a model instance.
    fn set_id(&mut self, oid: mongodb::oid::ObjectId) {
        self.id = Some(oid);
    }
}
# fn main() {}
```

### indexes
To manually declare indexes, declare them in the `indexes` method.

```rust
# #[macro_use]
# extern crate mongodb;
# extern crate serde;
# #[macro_use(Serialize, Deserialize)]
# extern crate serde_derive;
# extern crate wither;
#
use mongodb::coll::options::IndexModel;

# #[derive(Serialize, Deserialize, Debug, Clone)]
# pub struct User {
#     /// The user's unique ID.
#     #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
#     pub id: Option<mongodb::oid::ObjectId>,
# }
#
impl<'a> wither::Model<'a> for User {
    // snip ...
#
#     /// The name of this model's collection.
#     const COLLECTION_NAME: &'static str = "users";
#
#     /// Implement the getter for the ID of a model instance.
#     fn id(&self) -> Option<mongodb::oid::ObjectId> {
#         return self.id.clone();
#     }
#
#     /// Implement the setter for the ID of a model instance.
#     fn set_id(&mut self, oid: mongodb::oid::ObjectId) {
#         self.id = Some(oid);
#     }

    // Define any indexes which need to be maintained for your model here.
    fn indexes() -> Vec<IndexModel> {
        return vec![
            IndexModel{
                keys: doc!{"email": 1},
                // Args are: name, background, unique, ttl, sparse.
                options: wither::basic_index_options("unique-email", true, Some(true), None, None),
            },
        ];
    }
}
# fn main() {}
```

Whenever [`Model::sync`](./trait.Model.html#method.sync) is called, it will synchronize any
indexes defined in this method with the database. Any indexes which do not exist in the model
definition will be removed (barring the default index on `_id`).
