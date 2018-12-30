### underlying driver
If at any point in time you need direct access to the [underlying driver](https://docs.rs/mongodb/latest/mongodb/), it is always available. All of the `Model` interface methods take a handle to the database, which is part of the underlying driver. You can then use the [`Model::COLLECTION_NAME`](https://docs.rs/wither/latest/wither/model/trait.Model.html#associatedconstant.COLLECTION_NAME) to ensure you are accessing the correct collection. You can also use the various model convenience methods for serialization, such as the [`Model::instance_from_document`](https://docs.rs/wither/latest/wither/model/trait.Model.html#method.instance_from_document) method.

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
# use mongodb::{
#     Document, ThreadedClient,
#     coll::options::IndexModel,
#     db::ThreadedDatabase,
# };
#
#[derive(Model, Serialize, Deserialize)]
struct MyModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<mongodb::oid::ObjectId>,

    #[model(index(index="dsc", unique="true"))]
    pub email: String,
}

fn main() {
    // Your DB handle. This is part of the underlying driver.
    let db = mongodb::Client::with_uri("mongodb://localhost:27017/").unwrap().db("mydb");

    // Use the driver directly, but rely on your model for getting the collection name.
    let coll = db.collection(MyModel::COLLECTION_NAME);

    // Now you can use the raw collection interface to your heart's content.
}
```
